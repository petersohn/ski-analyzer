use ski_analyzer_lib::error::Result;
use ski_analyzer_lib::utils::gpx::load_from_file;

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let path = &args[1];
    eprintln!("{path}");
    let gpx: gpx::Gpx = load_from_file(path)?;

    for track in gpx.tracks {
        for segment in track.segments {
            println!("segment(&[");
            for wp in segment.points {
                let p = wp.point();
                println!("  [({}, {})],", p.x(), p.y());
            }
            println!("]),");
        }
    }

    Ok(())
}
