use clap::Parser;
use indicatif::{ProgressBar, ProgressStyle};
use std::fs::File;
use std::io::{self, BufRead, BufReader, Read};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use zip::result::ZipError;

/// Sebuah tool brute-force password ZIP yang cepat, akurat, dan kaya fitur, ditulis dalam Rust.
#[derive(Parser, Debug)]
#[command(version = "3.5.1", author = "Gemini AI", about, long_about = None)]
struct Cli {
    #[arg(short = 'f', long)]
    zip_file: PathBuf,
    #[arg(short = 'w', long)]
    wordlist: PathBuf,
    #[arg(short = 'o', long)]
    output_dir: Option<PathBuf>,
    #[arg(short = 't', long)]
    threads: Option<usize>,
}

fn count_lines(path: &Path) -> io::Result<u64> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    Ok(reader.lines().count() as u64)
}

fn main() -> io::Result<()> {
    let cli = Cli::parse();

    if !cli.zip_file.exists() {
        return Err(io::Error::new(io::ErrorKind::NotFound, "File ZIP tidak ditemukan."));
    }
    let total_passwords = count_lines(&cli.wordlist)?;
    if total_passwords == 0 {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "File wordlist kosong."));
    }

    let num_threads = cli.threads.unwrap_or_else(num_cpus::get);
    let (tx, rx) = std::sync::mpsc::channel::<String>();
    let job_receiver = Arc::new(Mutex::new(rx));
    let shutdown_flag = Arc::new(AtomicBool::new(false));
    let attempted_count = Arc::new(AtomicU64::new(0));

    println!("========================================");
    println!("    Rust ZIP Cracker v3.5.1 (Final)   ");
    println!("========================================");
    println!("[+] File Target     : {}", cli.zip_file.display());
    println!("[+] Wordlist        : {} ({} passwords)", cli.wordlist.display(), total_passwords);
    println!("[+] Threads         : {}", num_threads);
    if let Some(dir) = &cli.output_dir {
        println!("[+] Output Direktori : {}", dir.display());
        std::fs::create_dir_all(dir)?;
    }
    println!("----------------------------------------");
    
    let start_time = Instant::now();

    let pb = ProgressBar::new(total_passwords);
    pb.set_style(ProgressStyle::default_bar()
        .template("{spinner:.green} {wide_bar:.cyan/blue} {percent}% | {pos}/{len} | {msg}")
        .expect("Template style error")
        .progress_chars("=> "));

    let ui_shutdown_flag = shutdown_flag.clone();
    let ui_attempted_count = attempted_count.clone();
    let ui_thread = thread::spawn(move || {
        while !ui_shutdown_flag.load(Ordering::SeqCst) {
            let count = ui_attempted_count.load(Ordering::Relaxed);
            pb.set_position(count);
            let elapsed_secs = start_time.elapsed().as_secs_f64();
            if elapsed_secs > 0.0 {
                let rate = count as f64 / elapsed_secs;
                pb.set_message(format!("{:.0} pwd/s", rate));
            }
            thread::sleep(Duration::from_millis(100));
        }
        let count = ui_attempted_count.load(Ordering::Relaxed);
        pb.set_position(count);
        pb.finish_with_message("Selesai");
    });
    
    let ctrlc_shutdown_flag = shutdown_flag.clone();
    ctrlc::set_handler(move || {
        println!("\n\n[!] Sinyal interrupt diterima. Menghentikan proses secara anggun...");
        ctrlc_shutdown_flag.store(true, Ordering::SeqCst);
    }).expect("Error setting Ctrl-C handler");

    let mut handles = vec![];
    for _ in 0..num_threads {
        let receiver = Arc::clone(&job_receiver);
        let flag = Arc::clone(&shutdown_flag);
        let counter = Arc::clone(&attempted_count);
        let zip_path = cli.zip_file.clone();

        handles.push(thread::spawn(move || -> Option<String> {
            loop {
                if flag.load(Ordering::SeqCst) { break; }
                let password = match receiver.lock().unwrap().recv() {
                    Ok(p) => p,
                    Err(_) => break,
                };
                counter.fetch_add(1, Ordering::Relaxed);
                
                if try_password(&zip_path, &password) {
                    flag.store(true, Ordering::SeqCst);
                    return Some(password);
                }
            }
            None
        }));
    }

    let reader_thread = thread::spawn(move || {
        if let Ok(file) = File::open(cli.wordlist) {
            let reader = BufReader::new(file);
            for line in reader.lines().flatten() {
                if tx.send(line).is_err() { break; }
            }
        }
    });

    let mut found_password = None;
    for handle in handles {
        if let Some(password) = handle.join().unwrap_or(None) {
            found_password = Some(password);
        }
    }

    shutdown_flag.store(true, Ordering::SeqCst);
    reader_thread.join().unwrap();
    ui_thread.join().unwrap();
    
    print_final_report(&found_password, &attempted_count, total_passwords, start_time.elapsed());
    
    if let (Some(password), Some(output_dir)) = (found_password, cli.output_dir) {
        println!("\n[+] Memulai proses ekstraksi...");
        extract_zip(&cli.zip_file, &output_dir, &password)?;
        println!("[+] Proses ekstraksi selesai.");
    }

    Ok(())
}

fn try_password(zip_path: &Path, password: &str) -> bool {
    let file = match File::open(zip_path) {
        Ok(f) => f,
        Err(_) => return false,
    };
    let mut archive = match zip::ZipArchive::new(file) {
        Ok(a) => a,
        Err(_) => return false,
    };

    for i in 0..archive.len() {
        match archive.by_index_decrypt(i, password.as_bytes()) {
            Ok(mut zip_file) => {
                if matches!(zip_file.read(&mut [0; 1]), Ok(0) | Ok(1)) {
                    return true;
                }
            }
            Err(_) => continue,
        }
    }
    false
}


fn print_final_report(found_password: &Option<String>, attempted: &Arc<AtomicU64>, total: u64, elapsed: Duration) {
    let attempted_count = attempted.load(Ordering::Relaxed);
    println!("\n----------------------------------------");
    println!("[+] Proses Selesai!");
    println!("[+] Waktu Total       : {:.2?}", elapsed);
    println!("[+] Total Percobaan   : {} / {}", attempted_count, total);
    if elapsed.as_secs_f64() > 0.0 {
        let rate = attempted_count as f64 / elapsed.as_secs_f64();
        println!("[+] Kecepatan Rata-rata : {:.2} password/detik", rate);
    }
    println!("----------------------------------------");
    if let Some(password) = found_password {
        println!("\n✅ PASSWORD DITEMUKAN: {}", password);
    } else {
        println!("\n❌ Password tidak ditemukan di dalam wordlist.");
    }
}


fn extract_zip(zip_path: &Path, output_dir: &Path, password: &str) -> io::Result<()> {
    let file = File::open(zip_path)?;
    let mut archive = zip::ZipArchive::new(file)?;
    for i in 0..archive.len() {
        let mut file = match archive.by_index_decrypt(i, password.as_bytes()) {
            Ok(f) => f,
            Err(ZipError::UnsupportedArchive(_)) => continue,
            Err(e) => return Err(io::Error::new(io::ErrorKind::Other, e)),
        };
        
        let outpath = match file.enclosed_name() {
            Some(path) => output_dir.join(path),
            None => continue,
        };
        if (*file.name()).ends_with('/') {
            std::fs::create_dir_all(&outpath)?;
        } else {
            if let Some(p) = outpath.parent() {
                if !p.exists() { std::fs::create_dir_all(p)?; }
            }
            let mut outfile = File::create(&outpath)?;
            // --- INI ADALAH BARIS YANG DIPERBAIKI ---
            io::copy(&mut file, &mut outfile)?;
        }
    }
    Ok(())
}
