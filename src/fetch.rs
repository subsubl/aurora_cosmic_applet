use std::error::Error;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use reqwest::Client;
use directories::ProjectDirs;

pub const FORECAST_URL: &str = "https://services.swpc.noaa.gov/text/3-day-geomag-forecast.txt";
pub const VIEWLINE_URL: &str = "https://services.swpc.noaa.gov/experimental/images/aurora_dashboard/tonights_static_viewline_forecast.png";
pub const LATEST_URL: &str = "https://services.swpc.noaa.gov/images/animations/ovation/north/latest.jpg";
pub const KURL: &str = "https://services.swpc.noaa.gov/images/station-k-index.png";

#[derive(Clone, Debug)]
pub struct AuroraData {
    pub value: f32,
    pub _cache_dir: PathBuf,
}

pub async fn update_data() -> Result<AuroraData, Box<dyn Error>> {
    let client = Client::new();
    let cache_dir = get_cache_dir()?;
    fs::create_dir_all(&cache_dir)?;

    let forecast_path = cache_dir.join("aurora.txt");
    let viewline_path = cache_dir.join("aurora.png");
    let latest_path = cache_dir.join("aurora_latest.jpg");
    let kindex_path = cache_dir.join("aurora_kindex.png");
    let combo_path = cache_dir.join("aurora_combo.jpg");

    // Download files
    download_file(&client, FORECAST_URL, &forecast_path).await?;
    download_file(&client, VIEWLINE_URL, &viewline_path).await?;
    download_file(&client, LATEST_URL, &latest_path).await?;
    download_file(&client, KURL, &kindex_path).await?;

    // Parse forecast
    let forecast_content = fs::read_to_string(&forecast_path)?;
    let value = parse_forecast(&forecast_content)?;

    // Image processing (combine images)
    // Replicating: magick $latest $viewline -resize x512 +append $combo
    // We will do this in a separate function or crate, but for now let's just trigger it here if possible or return paths
    // For simplicity, we can do it here using `image` crate.
    combine_images(&latest_path, &viewline_path, &combo_path)?;

    Ok(AuroraData {
        value,
        _cache_dir: cache_dir,
    })
}

async fn download_file(client: &Client, url: &str, path: &Path) -> Result<(), Box<dyn Error>> {
    let response = client.get(url).send().await?;
    let bytes = response.bytes().await?;
    let mut file = fs::File::create(path)?;
    file.write_all(&bytes)?;
    Ok(())
}

fn get_cache_dir() -> Result<PathBuf, Box<dyn Error>> {
    // Mimic the user's $HOME/.cache
    if let Some(_proj_dirs) = ProjectDirs::from("", "", "aurora-applet") {
        // This usually gives ~/.config/aurora-applet, we want ~/.cache specifically if adhering strictly,
        // but using standard dirs is better.
        // User specifically used $HOME/.cache/aurora.txt. 
        // Let's stick to standard ~/.cache/aurora-applet/ or just ~/.cache if we want to be messy?
        // User script: $HOME/.cache/aurora.txt
        let home = std::env::var("HOME")?;
        Ok(PathBuf::from(home).join(".cache"))
    } else {
        Err("Could not find home directory".into())
    }
}

fn parse_forecast(content: &str) -> Result<f32, Box<dyn Error>> {
    // awk logic replication:
    // {a[NR]=$3; b=$2} END {val=b; for (i=NR-7; i<=NR-4; i++) if (a[i]>val) val=a[i]; print val}
    
    // We need to parse lines, store column 2 and 3.
    // Lines in the file are not just numbers, there's text.
    // The awk script assumes specific columns.
    
    let lines: Vec<&str> = content.lines().collect();
    let mut col2_val: f32 = 0.0;
    let mut col3_vals: Vec<f32> = Vec::new();

    // The awk script uses 1-based indexing for lines (NR).
    // It stores EVERY line's col3 in `a` (indexed by NR).
    // It updates `b` with EVERY line's col2.
    // So `b` becomes the col2 of the helper LAST line.
    
    for line in &lines {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 3 {
             if let Ok(v2) = parts[1].parse::<f32>() {
                 col2_val = v2;
             }
             // Store col3 if valid, or maybe store 0.0? Awk would store empty string or 0.
             // We need to track line numbers effectively.
             if let Ok(v3) = parts[2].parse::<f32>() {
                 col3_vals.push(v3);
             } else {
                 col3_vals.push(0.0); // Placeholder to keep index alignment
             }
        } else {
             col3_vals.push(0.0);
        }
    }
    
    let nr = lines.len();
    if nr < 8 {
        return Ok(0.0); // Not enough data
    }
    
    let mut val = col2_val;
    
    // Loop i from NR-7 to NR-4 (inclusive)
    // Arrays in awk are 1-based (usually matched NR), Rust vec is 0-based.
    // NR corresponding to lines[nr-1].
    // So NR-7 corresponds to index (nr-1)-7 = nr-8 ?
    // Let's trace:
    // NR=10. i runs 3 to 6.
    // Rust indices: 0..9.
    // partial line: lines[2] (NR=3)
    // So index = i - 1.
    
    let start_idx = (nr as isize - 7 - 1).max(0) as usize; // NR-7 => index
    let end_idx = (nr as isize - 4 - 1).max(0) as usize;   // NR-4 => index

    // Check bounds
    if start_idx < col3_vals.len() && end_idx < col3_vals.len() {
        for i in start_idx..=end_idx {
            if col3_vals[i] > val {
                val = col3_vals[i];
            }
        }
    }

    Ok(val)
}

fn combine_images(latest: &Path, viewline: &Path, output: &Path) -> Result<(), Box<dyn Error>> {
    // magick $latest $viewline -resize x512 +append $combo
    // We need to resize viewline to height 512? Or both?
    // "resize x512" typically means resize to height 512, width auto.
    // Then append horizontally (+append).
    
    let img_latest = image::open(latest)?;
    let img_viewline = image::open(viewline)?;
    
    // Resize viewline to height 512
    let new_height = 512;
    let nwidth_viewline = (img_viewline.width() as u32 * new_height) / img_viewline.height() as u32;
    let img_viewline_resized = img_viewline.resize_exact(nwidth_viewline, new_height, image::imageops::FilterType::Lanczos3);

    // Resize latest to height 512 as well for clean append?
    // The command `magick $latest $viewline -resize x512` applies resize to both if loaded before?
    // Actually magick syntax: loading images, then applying operator. It might apply to the sequence.
    // Let's assume we want both height 512.
    let nwidth_latest = (img_latest.width() as u32 * new_height) / img_latest.height() as u32;
    let img_latest_resized = img_latest.resize_exact(nwidth_latest, new_height, image::imageops::FilterType::Lanczos3);
    
    // Create new image
    let total_width = nwidth_latest + nwidth_viewline;
    let mut combo = image::RgbaImage::new(total_width, new_height);
    
    image::imageops::overlay(&mut combo, &img_latest_resized, 0, 0);
    image::imageops::overlay(&mut combo, &img_viewline_resized, nwidth_latest.into(), 0);
    
    combo.save(output)?;
    
    Ok(())
}
