use std::vec::Vec;
use std::collections::HashMap;
use basic_mutator::{Mutator, InputDatabase, Rng};
use std::string::String;
use glob::glob;
use std::fs::File;
use std::path::Path;
use std::sync::Arc;
use std::io::prelude::*;
use std::env;
use std::ffi::OsStr;

fn gen_seed() -> u64{

    unsafe {
        core::arch::x86_64::_rdtsc()
    }
}

pub struct Corpus {
    pub filenames: Vec<String>,
    pub files: Vec<Vec<u8>>,
    seed: u64,
    max_size: usize,
    rng: Rng,

    

    /// corpus directory path
    dirpath: String, 

    // Last used input
    last_input: Vec<u8>,

    last_input_filename: String,
}

impl Corpus{
    pub fn new(max_size: usize, seed: u64, mut dirpath: String) -> Self{

        // read in all the filenames recursively from `dirpath`
        dirpath.push_str("/**/*");
        let mut filenames:Vec<String> = Vec::new();
        let paths = glob(&dirpath).unwrap();
        for filename in paths{
            let fnm = filename.unwrap();
            let metadata = std::fs::metadata(&fnm)
                    .expect("couldn't read file metadata");
            if metadata.is_file(){
                filenames.push(fnm.as_path().display().to_string());
            }
        }


        let mut files: Vec<Vec<u8>> = Vec::new();
        for filename in filenames.iter(){
            let mut f = File::open(&filename).expect("no file found");
            let metadata = std::fs::metadata(&filename)
                .expect("couldn't read file metadata");
            let mut buffer: Vec<u8> = vec![0; metadata.len() as usize];
            buffer = std::fs::read(&filename).unwrap();

            files.push(buffer);
        }
        Corpus{
            filenames: filenames,
            files: files,
            seed: seed,
            max_size: max_size,
            rng: Rng {
                seed:         seed,
                exp_disabled: true,
            },
            last_input: Vec::new(),
            dirpath: dirpath,
            last_input_filename: String::from(""),
        }
    }

    // this might be slow due to all the clones
    // fix this
    pub fn mutate_rand(&mut self, threadid: i32) -> String{
        
        
        let seed: u64 = self.seed;
        let max_size = self.max_size;
        let (filename, x) = self.rand_file().unwrap();
        //print!("unmutated: {:#x?}\n", x);
        let mut mutator = Mutator::new().max_input_size(max_size).seed(seed);
        mutator.input = x;
        mutator.mutate(100, self);
        //print!("mutated: {:#x?}\n", mutator.input);
        self.last_input = mutator.input.clone();

        let extension = {
            let here = Path::new(&filename).extension();
            if here.is_none(){
                ""
            }
            else{
                here.unwrap().to_str().unwrap()
            }
        };
        let mut fname: String = String::from("");
        if extension == ""{
            fname = format!("{}\\mainpath_{}_{:?}",env::current_dir().unwrap().display() ,gen_seed(),threadid);
        }
        else{
            fname = format!("{}\\mainpath_{}_{:?}.{}",env::current_dir().unwrap().display() ,gen_seed(),threadid, extension);
        }


        let mut file = File::create(&fname).expect("couldnt create da file");
        //self.last_input_filename = String::from(&fname);
        //print!("inp: {}\n", fname);
        file.write_all(&self.last_input).expect("could not write to file");

        fname

    }
    pub fn lastinputfn(&mut self) -> String{
        self.last_input_filename.clone()
    }

    pub fn setlastinputfn(&mut self, nig: String){
        self.last_input_filename = nig;
    }


    pub fn lastinput(&mut self) -> &Vec<u8>{
        &self.last_input
    }

    fn rand_file(&mut self) -> Option<(String, Vec<u8>)> {
        let rand = self.rng.next() as usize % self.files.len();

        let retval = self.files[rand].clone();
        let fname = self.filenames[rand].clone();
        Some((fname, retval))
    }

    pub fn new_input(&mut self, input: Vec<u8>){
        self.files.push(input);
    }
}

impl InputDatabase for Corpus{
    fn num_inputs(&self) -> usize{
        self.files.len()
    }
    fn input(&self, idx: usize) -> Option<&[u8]>{
        let retval = self.files[idx].as_slice();
        Some(retval)
    }
}