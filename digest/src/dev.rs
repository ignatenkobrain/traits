use super::{Digest, Input, VariableOutput, ExtendableOutput, XofReader};
use core::fmt::Debug;


#[macro_export]
macro_rules! new_test {
    ($name:ident, $test_name:expr, $hasher:ty, $test_func:expr) => {
        #[test]
        fn $name() {
            let inputs = include_bytes!(
                concat!("data/", $test_name, ".inputs.bin"));
            let outputs = include_bytes!(
                concat!("data/", $test_name, ".outputs.bin"));
            let index = include_bytes!(
                concat!("data/", $test_name, ".index.bin"));

            // u16 (2 bytes); start + end (x2); input, output (x2)
            assert_eq!(index.len() % (2*2*2), 0, "invlaid index length");
            let mut hasher = <$hasher>::default();
            for (i, chunk) in index.chunks(2*2*2).enumerate() {
                // proper aligment is assumed here
                let mut idx = unsafe {
                    *(chunk.as_ptr() as *const [[u16; 2]; 2])
                };
                // convert to LE for BE machine
                for val in idx.iter_mut() {
                    for i in val.iter_mut() { *i = i.to_le(); }
                }
                let input = &inputs[(idx[0][0] as usize)..(idx[0][1] as usize)];
                let output = &outputs[
                    (idx[1][0] as usize)..(idx[1][1] as usize)];
                if let Some(desc) = $test_func(&mut hasher, input, output) {
                    panic!("\n\
                        Failed test №{}: {}\n\
                        input: [{}..{}]\t{:?}\n\
                        output: [{}..{}]\t{:?}\n",
                        i, desc,
                        idx[0][0], idx[0][1], input,
                        idx[1][0], idx[1][1], output,
                    );
                }
            }

        }
    }
}

pub fn digest_test<D>(hasher: &mut D, input: &[u8], output: &[u8])
    -> Option<&'static str>
    where D: Digest + Debug + Clone
{
    // Test that it works when accepting the message all at once
    hasher.input(input);
    if hasher.result().as_slice() != output {
        return Some("whole message");
    }

    // Test if reset works correctly
    hasher.input(input);
    if hasher.result().as_slice() != output {
        return Some("whole message after reset");
    }

    // Test that it works when accepting the message in pieces
    let len = input.len();
    let mut left = len;
    while left > 0 {
        let take = (left + 1) / 2;
        hasher.input(&input[len - left..take + len - left]);
        left = left - take;
    }
    if hasher.result().as_slice() != output {
        return Some("message in pieces");
    }

    // Test processing byte-by-byte
    for chunk in input.chunks(1) {
        hasher.input(chunk)
    }
    if hasher.result().as_slice() != output {
        return Some("message byte-by-byte");
    }
    None
}

pub fn xof_test<D>(hasher: &mut D, input: &[u8], output: &[u8])
    -> Option<&'static str>
    where D: Input + ExtendableOutput + Default + Debug + Clone
{
    let mut buf = [0u8; 1024];
    // Test that it works when accepting the message all at once
    hasher.process(input);

    {
        let out = &mut buf[..output.len()];
        hasher.xof_result().read(out);

        if out != output { return Some("whole message"); }
    }

    // Test if hasher resetted correctly
    hasher.process(input);

    {
        let out = &mut buf[..output.len()];
        hasher.xof_result().read(out);

        if out != output { return Some("whole message after reset"); }
    }


    // Test if hasher accepts message in pieces correctly
    let len = input.len();
    let mut left = len;
    while left > 0 {
        let take = (left + 1) / 2;
        hasher.process(&input[len - left..take + len - left]);
        left = left - take;
    }

    {
        let out = &mut buf[..output.len()];
        hasher.xof_result().read(out);
        if out != output { return Some("message in pieces"); }
    }

    // Test reading from reader byte by byte
    hasher.process(input);

    let mut reader = hasher.xof_result();
    let out = &mut buf[..output.len()];
    for chunk in out.chunks_mut(1) {
        reader.read(chunk);
    }

    if out != output { return Some("message in pieces"); }
    None
}

/*
#[derive(Debug, Clone)]
struct Dummy;

impl Input for Dummy {
    fn process(&mut self, _: &[u8]) {}
}

use generic_array::GenericArray;
use generic_array::typenum::U1;

impl super::FixedOutput for Dummy {
    type OutputSize = U1;
    fn fixed_result(&mut self) -> GenericArray<u8, U1> {
        Default::default()
    }
}

new_test!(dummy_test, "dummy", Dummy, digest_test);

*/
/*

pub struct Test {
    pub name: &'static str,
    pub input: &'static [u8],
    pub output: &'static [u8],
}

#[macro_export]
macro_rules! new_tests {
    [ $( $name:expr ),*  ] => {
        [$(
            Test {
                name: $name,
                input: include_bytes!(concat!("data/", $name, ".input.bin")),
                output: include_bytes!(concat!("data/", $name, ".output.bin")),
            },
        )*]
    };
    [ $( $name:expr ),+, ] => (new_tests![$($name),+])
}

pub fn run_digest_tests<D: Digest + Debug + Clone>(tests: &[Test]) {
    // Test that it works when accepting the message all at once
    for t in tests.iter() {
        let mut sh = D::default();
        sh.input(t.input);

        let out = sh.result();

        assert_eq!(out[..], t.output[..]);
    }

    // Test that it works when accepting the message in pieces
    for t in tests.iter() {
        let mut sh = D::default();
        let len = t.input.len();
        let mut left = len;
        while left > 0 {
            let take = (left + 1) / 2;
            sh.input(&t.input[len - left..take + len - left]);
            left = left - take;
        }

        let out = sh.result();

        assert_eq!(out[..], t.output[..]);
    }
}

pub fn run_variable_tests<D>(tests: &[Test])
    where D: Input + VariableOutput + Clone + Debug
{
    let mut buf = [0u8; 1024];
    // Test that it works when accepting the message all at once
    for t in tests.iter() {
        let mut sh = D::new(t.output.len()).unwrap();
        sh.process(t.input);

        let out = sh.variable_result(&mut buf[..t.output.len()]).unwrap();

        assert_eq!(out[..], t.output[..]);
    }

    // Test that it works when accepting the message in pieces
    for t in tests.iter() {
        let mut sh = D::new(t.output.len()).unwrap();
        let len = t.input.len();
        let mut left = len;
        while left > 0 {
            let take = (left + 1) / 2;
            sh.process(&t.input[len - left..take + len - left]);
            left = left - take;
        }

        let out = sh.variable_result(&mut buf[..t.output.len()]).unwrap();

        assert_eq!(out[..], t.output[..]);
    }
}


pub fn run_xof_tests<D>(tests: &[Test])
    where D: Input + ExtendableOutput + Default + Debug + Clone
{
    let mut buf = [0u8; 1024];
    // Test that it works when accepting the message all at once
    for t in tests.iter() {
        let mut sh = D::default();
        sh.process(t.input);

        let out = &mut buf[..t.output.len()];
        sh.xof_result().read(out);

        assert_eq!(out[..], t.output[..]);
    }

    // Test that it works when accepting the message in pieces
    for t in tests.iter() {
        let mut sh = D::default();
        let len = t.input.len();
        let mut left = len;
        while left > 0 {
            let take = (left + 1) / 2;
            sh.process(&t.input[len - left..take + len - left]);
            left = left - take;
        }

        let out = &mut buf[..t.output.len()];
        sh.xof_result().read(out);

        assert_eq!(out[..], t.output[..]);
    }

    // Test reeading from reader byte by byte
    for t in tests.iter() {
        let mut sh = D::default();
        sh.process(t.input);

        let mut reader = sh.xof_result();
        let out = &mut buf[..t.output.len()];
        for chunk in out.chunks_mut(1) {
            reader.read(chunk);
        }

        assert_eq!(out[..], t.output[..]);
    }
}
*/

pub fn one_million_a<D>(expected: &[u8])
    where D: Digest + Default + Debug + Clone
{
    let mut sh = D::default();
    for _ in 0..50_000 {
        sh.input(&[b'a'; 10]);
    }
    sh.input(&[b'a'; 500_000]);
    let out = sh.result();
    assert_eq!(out[..], expected[..]);
}


#[macro_export]
macro_rules! bench {
    ($name:ident, $engine:path, $bs:expr) => {
        #[bench]
        fn $name(b: &mut Bencher) {
            let mut d = <$engine>::default();
            let data = [0; $bs];

            b.iter(|| {
                d.input(&data);
            });

            b.bytes = $bs;
        }
    };

    ($engine:path) => {
        extern crate test;

        use test::Bencher;
        use digest::Digest;

        bench!(bench1_10,    $engine, 10);
        bench!(bench2_100,   $engine, 100);
        bench!(bench3_1000,  $engine, 1000);
        bench!(bench4_10000, $engine, 10000);
    }
}
