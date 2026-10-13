enum PieceState { 
    Missing, 
    InProgress, 
    Done 
}

struct PieceTable {
    pieces: Vec<PieceState>,
}

struct Config {
    info_hash: Vec<u8>,
    peer_id: String,
    num_pieces: usize,
}

struct PeerQueue {
    pending: VecDeque<String>,
    seen: HashSet<String>,
}