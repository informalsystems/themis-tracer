use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "themis-tracer",
    about = "Requirements traceability made easier"
)]
struct Opt {}

fn main() {
    let opt = Opt::from_args();
    println!("{:?}", opt);
}
