mod core;
use docopt::Docopt;

const USAGE: &'static str = "
Usage: pr1 [options] <path>

Options:
    -w, --wordcount
    -h <word>, --hardsearch <word>
    -s <word>, --softsearch <word>
    -?, --help
";

enum ArgsOption {
    Path,
    Usage,
    WordCount,
    SoftSearch,
    HardSearch
}

fn main() {
    let args = Docopt::new(USAGE)
        .and_then(|dopt| dopt.parse())
        .unwrap_or_else(|e| e.exit());
        println!("{:?}", args);

    for arg in args.{

    }

    // match args{
        
    // }

    // if args.get_bool("<path>") { core::print(args.get_str("<path>")) }
    
    // if args.get_bool("-?") { println!({USAGE}) }

    // if  args.get_bool("-w") { core::word_count(args.get_str("<path>")) }

    // if args.get_bool("-h"){ core::hard_search(args.get_str("<path>"), args.get_str("-h"))}

    // if args.get_bool("-s"){ core::soft_search(args.get_str("<path>"), args.get_str("-s"))}  
}
