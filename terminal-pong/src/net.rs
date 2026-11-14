//! Wire format: every protobuf message is preceded by its length as a
//! 4-byte big-endian integer. TCP is a byte stream, so without the prefix
//! two messages sent close together can arrive glued into one read (or one
//! message can arrive split across two) and fail to decode.

use std::io::{self, Read, Write};

use protobuf::Message;

use crate::physics::Game;
use crate::proto::message::Game_State;

/// Far larger than any message this game sends; guards against garbage input
/// making us allocate gigabytes.
pub const MAX_FRAME: usize = 64 * 1024;

pub fn write_frame<W: Write, M: Message>(out: &mut W, msg: &M) -> io::Result<()> {
    let body = msg
        .write_to_bytes()
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    if body.len() > MAX_FRAME {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "message too large"));
    }
    // One write call, so a frame is never interleaved with another writer's.
    let mut frame = Vec::with_capacity(4 + body.len());
    frame.extend_from_slice(&(body.len() as u32).to_be_bytes());
    frame.extend_from_slice(&body);
    out.write_all(&frame)?;
    out.flush()
}

pub fn read_frame<R: Read, M: Message>(input: &mut R) -> io::Result<M> {
    let mut len = [0u8; 4];
    input.read_exact(&mut len)?;
    let len = u32::from_be_bytes(len) as usize;
    if len > MAX_FRAME {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "frame too large"));
    }
    let mut body = vec![0u8; len];
    input.read_exact(&mut body)?;
    M::parse_from_bytes(&body).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}

pub fn to_wire(game: &Game) -> Game_State {
    let mut s = Game_State::new();
    s.bx = game.bx;
    s.by = game.by;
    s.bdx = game.vx;
    s.bdy = game.vy;
    s.p1y = game.p1y.max(0) as u32;
    s.p2y = game.p2y.max(0) as u32;
    s.p1points = game.p1points;
    s.p2points = game.p2points;
    s.tick = game.tick;
    s.rally = game.rally;
    s.serve_in = game.serve_in;
    s
}

pub fn from_wire(s: &Game_State) -> Game {
    Game {
        bx: s.bx,
        by: s.by,
        vx: s.bdx,
        vy: s.bdy,
        p1y: s.p1y as i32,
        p2y: s.p2y as i32,
        p1points: s.p1points,
        p2points: s.p2points,
        rally: s.rally,
        tick: s.tick,
        serve_in: s.serve_in,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::proto::message::{ClientInput, ServerFrame};
    use protobuf::MessageField;
    use std::io::Cursor;

    /// Hands out at most one byte per read, like a badly fragmented socket.
    struct Trickle(Cursor<Vec<u8>>);
    impl Read for Trickle {
        fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
            let n = buf.len().min(1);
            self.0.read(&mut buf[..n])
        }
    }

    fn sample() -> Game {
        let mut g = Game::new();
        g.bx = 12.5;
        g.by = 7.25;
        g.vx = -31.0;
        g.vy = 4.5;
        g.p1y = 3;
        g.p2y = 17;
        g.p1points = 4;
        g.p2points = 9;
        g.rally = 6;
        g.tick = 1234;
        g.serve_in = 2;
        g
    }

    #[test]
    fn game_state_survives_the_wire() {
        let game = sample();
        assert_eq!(from_wire(&to_wire(&game)), game);
    }

    #[test]
    fn messages_glued_together_are_read_back_separately() {
        let mut bytes = Vec::new();
        for dy in [-1, 1, 0, -1] {
            let mut input = ClientInput::new();
            input.dy = dy;
            write_frame(&mut bytes, &input).unwrap();
        }
        let mut frame = ServerFrame::new();
        frame.you = 2;
        frame.state = MessageField::some(to_wire(&sample()));
        write_frame(&mut bytes, &frame).unwrap();

        let mut reader = Cursor::new(bytes);
        for dy in [-1, 1, 0, -1] {
            // dy = 0 encodes to an empty body, which a length prefix still delimits.
            assert_eq!(read_frame::<_, ClientInput>(&mut reader).unwrap().dy, dy);
        }
        let back: ServerFrame = read_frame(&mut reader).unwrap();
        assert_eq!(back.you, 2);
        assert_eq!(from_wire(&back.state), sample());
        assert_eq!(
            read_frame::<_, ClientInput>(&mut reader).unwrap_err().kind(),
            io::ErrorKind::UnexpectedEof
        );
    }

    #[test]
    fn a_message_split_across_many_reads_is_reassembled() {
        let mut bytes = Vec::new();
        let mut frame = ServerFrame::new();
        frame.winner = 1;
        frame.state = MessageField::some(to_wire(&sample()));
        write_frame(&mut bytes, &frame).unwrap();

        let back: ServerFrame = read_frame(&mut Trickle(Cursor::new(bytes))).unwrap();
        assert_eq!(back.winner, 1);
        assert_eq!(from_wire(&back.state), sample());
    }

    #[test]
    fn oversized_and_truncated_frames_are_rejected() {
        let huge = ((MAX_FRAME + 1) as u32).to_be_bytes().to_vec();
        assert_eq!(
            read_frame::<_, ClientInput>(&mut Cursor::new(huge)).unwrap_err().kind(),
            io::ErrorKind::InvalidData
        );
        let short = vec![0, 0, 0, 10, 1, 2, 3];
        assert_eq!(
            read_frame::<_, ClientInput>(&mut Cursor::new(short)).unwrap_err().kind(),
            io::ErrorKind::UnexpectedEof
        );
    }
}
