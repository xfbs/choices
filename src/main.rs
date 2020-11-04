use rand_distr::{Distribution, Normal};
use rand::thread_rng;
use structopt::StructOpt;
use threadpool::ThreadPool;
use std::sync::mpsc::channel;

#[derive(StructOpt, Debug, Clone)]
#[structopt(name = "choices")]
struct Opt {
    #[structopt(long, default_value = "100")]
    mean: f64,

    #[structopt(long, default_value = "15")]
    dev: f64,

    #[structopt(long, default_value = "100")]
    choices: usize,

    #[structopt(long, default_value = "0.67")]
    skip: f64,

    #[structopt(long, default_value = "100")]
    runs: usize,

    #[structopt(long, default_value = "1")]
    threads: usize,
}

fn run(opt: &Opt) -> anyhow::Result<f64> {
    let mut rng = thread_rng();
    let normal = Normal::new(opt.mean, opt.dev)?;

    // how many choices are we just gonna skip?
    let skip_amount = (opt.choices as f64 * opt.skip).floor() as usize;

    // in the values that we skipped, what was the highest?
    let skip_max = normal
        .sample_iter(rng)
        .take(skip_amount)
        .fold(0./0., f64::max);

    // how many choices can we make?
    let choices = opt.choices - skip_amount - 1;

    // choose the next item that is better than the best we had
    let choice = normal
        .sample_iter(rng)
        .take(choices)
        .find(|n| *n > skip_max);

    let last = normal.sample(&mut rng);

    // if we didn't find any, take whatever was the last one.
    let choice = choice
        .unwrap_or(last);

    Ok(choice)
}

fn main() -> anyhow::Result<()> {
    //let mut rng = thread_rng();
    let opt = Opt::from_args();

    // thread pool
    let pool = ThreadPool::new(opt.threads);

    // responses
    let (tx, rx) = channel();

    let opt = Box::new(opt);
    for _ in 0..opt.runs {
        let tx = tx.clone();
        let opt = opt.clone();
        pool.execute(move || {
            let ret = run(&opt).unwrap();
            tx.send(ret).unwrap();
        });
    }

    let results: average::Variance = rx.iter().take(opt.runs).collect();

    println!("mean = {} deviation = {}", results.mean(), results.sample_variance().sqrt());

    Ok(())
}
