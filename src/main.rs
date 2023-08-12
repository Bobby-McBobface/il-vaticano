use std::{env, fs::File, io, time::Instant};

use pgn_reader::{BufferedReader, SanPlus, Skip, Visitor};

use shakmaty::{Bitboard, Chess, Color, Position, Rank, Role};

#[derive(Debug)]
struct IlVaticanoCounter {
    games: usize,
    sans: usize,
    ilvaticanos: usize,
    pos: Chess,
    time: Instant,
    passed: usize,
}

impl IlVaticanoCounter {
    fn new() -> IlVaticanoCounter {
        IlVaticanoCounter {
            games: 0,
            sans: 0,
            ilvaticanos: 0,
            pos: Chess::default(),
            time: Instant::now(),
            passed: 0,
        }
    }

    fn make_move(&mut self, san: SanPlus) {
        let m = san
            .san
            .to_move(&self.pos)
            .expect("Lichess should return only valid moves");
        self.pos.play_unchecked(&m);
    }
}

impl Visitor for IlVaticanoCounter {
    type Result = ();
    fn begin_game(&mut self) {
        self.pos = Chess::default();
    }

    fn san(&mut self, san: SanPlus) {
        self.sans += 1;

        // The D, E, F files, excluding the backrank. At least one bishop needs to be there for il vaticano to happen
        const BISHOP_MASK: Bitboard = Bitboard(15824412808329216);

        if (self.pos.our(Role::Bishop) & BISHOP_MASK) == Bitboard(0) {
            self.make_move(san);
            return;
        }

        let mut passed_ranks_check = false;
        for rank in 1..=6 {
            if (self.pos.our(Role::Bishop) & Bitboard::from_rank(Rank::new(rank))).count() >= 2
                && (self.pos.their(Role::Pawn) & Bitboard::from_rank(Rank::new(rank))).count() >= 2
            {
                passed_ranks_check = true;
                break;
            }
        }

        if !passed_ranks_check {
            self.make_move(san);
            return;
        }

        self.passed += 1;

        let fen = self.pos.board().board_fen(Bitboard(0)).to_string();

        if self.pos.turn() == Color::Black && fen.contains("bPPb")
            || self.pos.turn() == Color::White && fen.contains("BppB")
        {
            self.ilvaticanos += 1;
            // println!("found an il vaticano on game {} {}", self.games, self.site);
        }

        self.make_move(san);
    }

    fn begin_variation(&mut self) -> Skip {
        Skip(true) // stay in the mainline
    }

    fn end_game(&mut self) {
        self.games += 1;
        if self.games % 100_000 == 0 {
            println!(
                "{} games, {} il vaticanos, {} positions, {} passed, {:.5}% positions {:.5}% games",
                self.games,
                self.ilvaticanos,
                self.sans,
                self.passed,
                (self.ilvaticanos as f32 / self.sans as f32) * 100.0,
                (self.ilvaticanos as f32 / self.games as f32) * 100.0
            );
            let elapsed_time = self.time.elapsed();
            println!("Took {} ms.", elapsed_time.as_millis());
        }
    }
}

fn main() -> Result<(), io::Error> {
    let now = Instant::now();

    for arg in env::args().skip(1) {
        let file = File::open(&arg)?;

        let uncompressed: Box<dyn io::Read> = Box::new(zstd::Decoder::new(file)?);
        let mut reader = BufferedReader::new(uncompressed);

        let mut stats = IlVaticanoCounter::new();
        reader.read_all(&mut stats)?;
        println!("{}: {:?}", arg, stats);
    }

    let elapsed_time = now.elapsed();
    println!("Took {} ms.", elapsed_time.as_millis());

    Ok(())
}
