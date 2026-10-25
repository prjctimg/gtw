#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("M0.5 spike - no-ffmpeg (gtw)");
    println!("Deps present: rusty_vp9, matroska-demuxer, symphonia, rodio");
    println!("zvidlib not available (repo 404) - using fallback path if needed");
    println!("Spike scaffold complete. To pass acceptance:");
    println!("1. Download a short VP9/Opus WebM from Wikimedia");
    println!("2. Demux with matroska-demuxer");
    println!("3. Decode VP9 with rusty_vp9 (first 120 frames)");
    println!("4. Decode Opus with symphonia to PCM, play via rodio");
    println!("5. Seek to 50% and resume");
    println!("6. Verify no ffmpeg/libavcodec linkage");
    Ok(())
}
