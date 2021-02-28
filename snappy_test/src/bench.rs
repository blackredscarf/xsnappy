use std::process::{Command, Stdio};
use std::io::{BufWriter, Write, Read};
use std::fs::File;
use csnappy;
use xsnappy::{max_encode_len, encode};
use xsnappy::{decode_len, decode};
use criterion::{criterion_group, criterion_main, Criterion, Bencher, Throughput, BenchmarkGroup};
use std::time::Duration;
use crate::golden::read_file_to_vec;
use snap::raw::{Encoder, Decoder};

pub static TEST_FILES: [(&str, &str, i32); 12] = [
    ("html", "html", 0),
    ("urls", "urls.10K", 0),
    ("jpg", "fireworks.jpeg", 0),
    ("jpg_200", "fireworks.jpeg", 200),
    ("pdf", "paper-100k.pdf", 0),
    ("html4", "html_x_4", 0),
    ("txt1", "alice29.txt", 0),
    ("txt2", "asyoulik.txt", 0),
    ("txt3", "lcet10.txt", 0),
    ("txt4", "plrabn12.txt", 0),
    ("pb", "geo.protodata", 0),
    ("gaviota", "kppkn.gtb", 0),
];

const HTML: &'static [u8] = include_bytes!("../testdata/bench/html");
const URLS: &'static [u8] = include_bytes!("../testdata/bench/urls.10K");
const JPG: &'static [u8] = include_bytes!("../testdata/bench/fireworks.jpeg");
const JPG200: &'static [u8] = include_bytes!("../testdata/bench/fireworks.jpeg");
const PDF: &'static [u8] = include_bytes!("../testdata/bench/paper-100k.pdf");
const HTML4: &'static [u8] = include_bytes!("../testdata/bench/html_x_4");
const TEXT1: &'static [u8] = include_bytes!("../testdata/bench/alice29.txt");
const TEXT2: &'static [u8] = include_bytes!("../testdata/bench/asyoulik.txt");
const TEXT3: &'static [u8] = include_bytes!("../testdata/bench/lcet10.txt");
const TEXT4: &'static [u8] = include_bytes!("../testdata/bench/plrabn12.txt");
const PB: &'static [u8] = include_bytes!("../testdata/bench/geo.protodata");
const GAVIOTA: &'static [u8] = include_bytes!("../testdata/bench/kppkn.gtb");


pub fn run_command(cmd: &str, arg: &str, data: &[u8]) -> Vec<u8> {
    let mut p = Command::new(cmd)
        .arg(arg)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn().unwrap();

    let stdin = p.stdin.as_mut().unwrap();
    stdin.write(data);

    let output = p.wait_with_output().unwrap();
    return output.stdout
}

pub fn csnappy_encode(src: &[u8]) -> Vec<u8> {
    let max_len = max_encode_len(src.len());
    let mut dst = Vec::<u8>::with_capacity(max_len);
    dst.resize(max_len, 0);
    let size = csnappy::compress(src, &mut dst).unwrap();
    dst.resize(size, 0);
    return dst
}

pub fn csnappy_decode(src: &[u8]) -> Vec<u8> {
    let dec_len = decode_len(src).unwrap_or(0);
    let mut dst = Vec::<u8>::with_capacity(dec_len);
    dst.resize(dec_len, 0);
    let size = csnappy::decompress(src, &mut dst).unwrap();
    dst.resize(size, 0);
    return dst
}

pub fn rsnappy_encode(src: &[u8]) -> Vec<u8> {
    let max_len = max_encode_len(src.len());
    let mut dst = Vec::<u8>::with_capacity(max_len);
    dst.resize(max_len, 0);
    let size = encode(&mut dst, src);
    dst.resize(size, 0);
    return dst
}

pub fn rsnappy_decode(src: &[u8]) -> Vec<u8> {
    let dec_len = decode_len(src).unwrap_or(0);
    let mut dst = Vec::<u8>::with_capacity(dec_len);
    dst.resize(dec_len, 0);
    let size = decode(&mut dst, src).unwrap_or(0);
    dst.resize(size, 0);
    return dst
}

pub fn snap_encode(src: &[u8]) -> Vec<u8> {
    let max_len = max_encode_len(src.len());
    let mut dst = Vec::<u8>::with_capacity(max_len);
    dst.resize(max_len, 0);
    let size = Encoder::new().compress(src, &mut dst).unwrap();
    dst.resize(size, 0);
    return dst
}

pub fn snap_decode(src: &[u8]) -> Vec<u8> {
    let dec_len = decode_len(src).unwrap_or(0);
    let mut dst = Vec::<u8>::with_capacity(dec_len);
    dst.resize(dec_len, 0);
    let size = Decoder::new().decompress(src, &mut dst).unwrap_or(0);
    dst.resize(size, 0);
    return dst
}

pub fn run_bench<F>(c: &mut Criterion, group_name: &str,
                 bench_name: &str, src: &[u8], bench_fn: F) where F: Fn(&mut Bencher) {
    let data_len = Throughput::Bytes(src.len() as u64);
    let mut group = c.benchmark_group(group_name);
    group.throughput(data_len);     // It must be set before bench_function
    group.bench_function(bench_name, bench_fn);
    group.sample_size(50);
    group.warm_up_time(Duration::from_millis(500));
    group.measurement_time(Duration::from_secs(3));
    group.finish()
}

type ProcessFn = fn(&[u8]) -> Vec<u8>;
pub fn compress(c: &mut Criterion, group: &str, com_fn: ProcessFn,
                name: &str, mut src: &[u8], size: usize) {
    if size > 0 {
        src = &src[..size];
    }

    run_bench(c, group,
              &format!("compress/{}", name), src,
            move |b| {
                b.iter(|| {
                    com_fn(src)
                })
            })
}

pub fn decompress(c: &mut Criterion, group: &str, dec_fn: ProcessFn,
                  name: &str, mut src: &[u8], size: usize) {
    if size > 0 {
        src = &src[..size];
    }
    let compressed = rsnappy_encode(src);
    run_bench(c, group,
              &format!("decompress/{}", name), src,
              move |b| {
                  b.iter(|| {
                      dec_fn(&compressed)
                  })
              })
}

fn rust_benches(c: &mut Criterion) {
    // compress(c, "rust", rsnappy_encode, "zflat00_html", HTML, 0);
    // compress(c, "rust", rsnappy_encode, "zflat01_urls", URLS, 0);
    // compress(c, "rust", rsnappy_encode, "zflat02_jpg", JPG, 0);
    // compress(c, "rust", rsnappy_encode, "zflat03_jpg_200", JPG200, 200);
    // compress(c, "rust", rsnappy_encode, "zflat04_pdf", PDF, 0);
    // compress(c, "rust", rsnappy_encode, "zflat05_html4", HTML4, 0);
    // compress(c, "rust", rsnappy_encode, "zflat06_txt1", TEXT1, 0);
    // compress(c, "rust", rsnappy_encode, "zflat07_txt2", TEXT2, 0);
    // compress(c, "rust", rsnappy_encode, "zflat08_txt3", TEXT3, 0);
    // compress(c, "rust", rsnappy_encode, "zflat09_txt4", TEXT4, 0);
    // compress(c, "rust", rsnappy_encode, "zflat10_pb", PB, 0);
    // compress(c, "rust", rsnappy_encode, "zflat11_gaviota", GAVIOTA, 0);

    // decompress(c, "rust", rsnappy_decode, "uflat00_html", HTML, 0);
    decompress(c, "rust", rsnappy_decode, "uflat01_urls", URLS, 0);
    decompress(c, "rust", rsnappy_decode, "uflat02_jpg", JPG, 0);
    // decompress(c, "rust", rsnappy_decode, "uflat03_jpg_200", JPG200, 200);
    // decompress(c, "rust", rsnappy_decode, "uflat04_pdf", PDF, 0);
    // decompress(c, "rust", rsnappy_decode, "uflat05_html4", HTML4, 0);
    // decompress(c, "rust", rsnappy_decode, "uflat06_txt1", TEXT1, 0);
    // decompress(c, "rust", rsnappy_decode, "uflat07_txt2", TEXT2, 0);
    // decompress(c, "rust", rsnappy_decode, "uflat08_txt3", TEXT3, 0);
    // decompress(c, "rust", rsnappy_decode, "uflat09_txt4", TEXT4, 0);
    // decompress(c, "rust", rsnappy_decode, "uflat10_pb", PB, 0);
    // decompress(c, "rust", rsnappy_decode, "uflat11_gaviota", GAVIOTA, 0);
}


fn cpp_benches(c: &mut Criterion) {
    compress(c, "cpp", csnappy_encode, "zflat00_html", HTML, 0);
    compress(c, "cpp", csnappy_encode, "zflat01_urls", URLS, 0);
    compress(c, "cpp", csnappy_encode, "zflat02_jpg", JPG, 0);
    compress(c, "cpp", csnappy_encode, "zflat03_jpg_200", JPG200, 200);
    compress(c, "cpp", csnappy_encode, "zflat04_pdf", PDF, 0);
    compress(c, "cpp", csnappy_encode, "zflat05_html4", HTML4, 0);
    compress(c, "cpp", csnappy_encode, "zflat06_txt1", TEXT1, 0);
    compress(c, "cpp", csnappy_encode, "zflat07_txt2", TEXT2, 0);
    compress(c, "cpp", csnappy_encode, "zflat08_txt3", TEXT3, 0);
    compress(c, "cpp", csnappy_encode, "zflat09_txt4", TEXT4, 0);
    compress(c, "cpp", csnappy_encode, "zflat10_pb", PB, 0);
    compress(c, "cpp", csnappy_encode, "zflat11_gaviota", GAVIOTA, 0);

    decompress(c, "cpp", csnappy_decode, "uflat00_html", HTML, 0);
    decompress(c, "cpp", csnappy_decode, "uflat01_urls", URLS, 0);
    decompress(c, "cpp", csnappy_decode, "uflat02_jpg", JPG, 0);
    decompress(c, "cpp", csnappy_decode, "uflat03_jpg_200", JPG200, 200);
    decompress(c, "cpp", csnappy_decode, "uflat04_pdf", PDF, 0);
    decompress(c, "cpp", csnappy_decode, "uflat05_html4", HTML4, 0);
    decompress(c, "cpp", csnappy_decode, "uflat06_txt1", TEXT1, 0);
    decompress(c, "cpp", csnappy_decode, "uflat07_txt2", TEXT2, 0);
    decompress(c, "cpp", csnappy_decode, "uflat08_txt3", TEXT3, 0);
    decompress(c, "cpp", csnappy_decode, "uflat09_txt4", TEXT4, 0);
    decompress(c, "cpp", csnappy_decode, "uflat10_pb", PB, 0);
    decompress(c, "cpp", csnappy_decode, "uflat11_gaviota", GAVIOTA, 0);
}

pub fn snap_benches(c: &mut Criterion) {
    compress(c, "snap", snap_encode, "zflat00_html", HTML, 0);
    compress(c, "snap", snap_encode, "zflat01_urls", URLS, 0);
    compress(c, "snap", snap_encode, "zflat02_jpg", JPG, 0);
    compress(c, "snap", snap_encode, "zflat03_jpg_200", JPG200, 200);
    compress(c, "snap", snap_encode, "zflat04_pdf", PDF, 0);
    compress(c, "snap", snap_encode, "zflat05_html4", HTML4, 0);
    compress(c, "snap", snap_encode, "zflat06_txt1", TEXT1, 0);
    compress(c, "snap", snap_encode, "zflat07_txt2", TEXT2, 0);
    compress(c, "snap", snap_encode, "zflat08_txt3", TEXT3, 0);
    compress(c, "snap", snap_encode, "zflat09_txt4", TEXT4, 0);
    compress(c, "snap", snap_encode, "zflat10_pb", PB, 0);
    compress(c, "snap", snap_encode, "zflat11_gaviota", GAVIOTA, 0);

    decompress(c, "snap", snap_decode, "uflat00_html", HTML, 0);
    decompress(c, "snap", snap_decode, "uflat01_urls", URLS, 0);
    decompress(c, "snap", snap_decode, "uflat02_jpg", JPG, 0);
    decompress(c, "snap", snap_decode, "uflat03_jpg_200", JPG200, 200);
    decompress(c, "snap", snap_decode, "uflat04_pdf", PDF, 0);
    decompress(c, "snap", snap_decode, "uflat05_html4", HTML4, 0);
    decompress(c, "snap", snap_decode, "uflat06_txt1", TEXT1, 0);
    decompress(c, "snap", snap_decode, "uflat07_txt2", TEXT2, 0);
    decompress(c, "snap", snap_decode, "uflat08_txt3", TEXT3, 0);
    decompress(c, "snap", snap_decode, "uflat09_txt4", TEXT4, 0);
    decompress(c, "snap", snap_decode, "uflat10_pb", PB, 0);
    decompress(c, "snap", snap_decode, "uflat11_gaviota", GAVIOTA, 0);
}

pub fn run_all_benches(c: &mut Criterion) {
    rust_benches(c);
    // cpp_benches(c);
    // snap_benches(c);
}

#[cfg(test)]
mod tests {
    use xsnappy::{max_encode_len, encode};
    use criterion::Criterion;
    use crate::bench::{csnappy_encode, TEST_FILES};
    use crate::golden::{cmp, read_file_to_vec};

    fn comp_with_standard(src: &[u8]) {
        let max_len = max_encode_len(src.len());
        let mut dst = Vec::<u8>::with_capacity(max_len);
        dst.resize(max_len, 0);
        let got_len = encode(&mut dst, src);
        dst.resize(got_len, 0);

        let want = csnappy_encode(src);

        assert!(cmp(&dst, &want));
    }

    fn standard_test() {
        for (kind, name, size) in TEST_FILES.iter() {
            let data = read_file_to_vec(&format!("testdata/bench/{}", *name));
            comp_with_standard(&data);
        }
    }

    #[test]
    fn it_works() {
        standard_test();
    }
}