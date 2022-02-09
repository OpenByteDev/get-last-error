#![feature(test)]

extern crate test;

#[cfg(test)]
mod tests {
    use test::Bencher;

    #[bench]
    fn bench_get_last_error(b: &mut Bencher) {
        b.iter(|| get_last_error::Win32Error::new(0).to_string());
    }

    #[bench]
    fn bench_w32_error(b: &mut Bencher) {
        b.iter(|| w32_error::W32Error::new(0).to_string());
    }
    
    #[bench]
    fn bench_rust_win32error(b: &mut Bencher) {
        b.iter(|| rust_win32error::Win32Error::from(0).to_string());
    }
}
