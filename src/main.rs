extern crate debugger;
extern crate glob;
extern crate window;
extern crate corpus;

pub mod mesofile;
use std::path::Path;
use corpus::Corpus;
use debugger::{ Debugger, ExitType};
use std::sync::{Mutex,Arc};
use std::fs::File;
use std::io::prelude::*;
use std::mem::drop;



/// Routine to invoke on module loads
fn modload_handler(dbg: &mut Debugger, modname: &str, base: usize) {
    // Calculate what the filename for a cached meso would be for this module
    let path = mesofile::compute_cached_meso_name(dbg, modname, base);

    // Attempt to load breakpoints from the meso file
    mesofile::load_meso(dbg, &path);
}
fn gen_seed() -> u64{

    unsafe {
        core::arch::x86_64::_rdtsc()
    }
}





//get get_dbg to place only the breakpoints that were not removed
fn get_dbg(dbger: Option<Debugger>) -> Debugger<'static>{
    let  pid: Option<u32> = None;
    let  frequency_mode_enabled = false;
    let  verbose_mode_enabled = false;
    let  follow_fork_enabled = false;
    let  print_breakpoints_enabled = false;
    let  mesos: Vec<&Path> = Vec::new();



    let mut dbg: Debugger;
    if pid.is_none() {
        dbg = Debugger::spawn_proc(&["C:\\windows\\system32\\notepad.exe".into()], follow_fork_enabled);
        //dbg = Debugger::spawn_proc(&["C:\\Program Files\\Adobe\\Adobe Photoshop 2020\\Photoshop.exe".into()], follow_fork_enabled);
        //dbg = Debugger::spawn_proc(&["E:\\fuzzing\\git_gdifuzzer\\foxit-harness\\foxit_10\\foxit10.exe".into()], follow_fork_enabled);
        //dbg = Debugger::spawn_proc(&["C:\\Program Files\\Adobe\\Adobe Photoshop 2020\\Photoshop.exe".into()], follow_fork_enabled);
    }
    else {
        dbg = Debugger::attach(pid.unwrap() as u32);
    }
    

    // Attach to process
    dbg.set_always_freq(frequency_mode_enabled);
    dbg.set_verbose(verbose_mode_enabled);
    dbg.set_bp_print(print_breakpoints_enabled);
    if dbger.is_none() {
        for mesofile in mesos {
            mesofile::load_meso(&mut dbg, mesofile);
        }
        // Register callback routine for module loads so we can attempt to apply
        // breakpoints to it from the meso file cache
        dbg.register_modload_callback(Box::new(modload_handler));
    }
    else{
        let mut old_dbg = dbger.unwrap();

        dbg.coverage = old_dbg.coverage.clone();
        dbg.target_breakpoints =  old_dbg.target_breakpoints.clone();
        dbg.minmax_breakpoint = old_dbg.minmax_breakpoint.clone();
    }

   

    //dbg.register_breakpoint("Photoshop.exe", 0x2222, "funcnigger", 10, BreakpointType::Single, );
    dbg
}


fn main() 

{
    
    // photoshop setup
    print!("hi");
    
    let mut dbg = get_dbg(None);
    let dirpath = String::from("corpus");
    print!("aaa");
    let mut corpus = Arc::new(Mutex::new(Corpus::new(1024*1024*10, gen_seed(),dirpath)));
    print!("got dbg");

    
    
    //threads(corpus.clone());
    
    loop 
    {
        

        //print!("bafasfaasfoi sssadasda\n");
        let exit_code = dbg.run(corpus.clone());
        
        
        //print!("bafasffgfsgsdaoi sssadasda\n");
        match exit_code {
            ExitType::Crash(filename) => {
                // a crash happened, save off the last input and restart the process
                let mut x = corpus.lock().unwrap();
                let lastinputfn = x.lastinputfn();
                let extension = {
                    let here = Path::new(&lastinputfn).extension();
                    if here.is_none(){
                        ""
                    }
                    else{
                        here.unwrap().to_str().unwrap()
                    }
                };
                let fname = format!("{}.input.{}", filename, extension);
                

                let mut file = File::create(&fname).expect("couldnt create da file");
                file.write_all(x.lastinput()).expect("could not write to file");
                drop(x);
                print!("CRASH: {}", fname);
                
                //get get_dbg to place only the breakpoints that were not removed
                dbg = get_dbg(Some(dbg));
                
            }
            ExitType::ExitCode(6000) =>{
                return ();
            }
            ExitType::ExitCode(_) => {
                if dbg.had_new_coverage{
                    let mut corp = corpus.lock().unwrap();
                    let lastfn = corp.lastinputfn();
                    let lastinput = corp.lastinput().clone();
                    
                    corp.filenames.push(lastfn);
                    corp.files.push(lastinput);
                }
                //get get_dbg to place only the breakpoints that were not removed
                dbg = get_dbg(Some(dbg));
            }
            
        }
        print!("finishing loop lolz");
    }

    //print!("{}", window::ifexit())
}



