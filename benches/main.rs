mod benchmarks;
mod util;

fn main() {
    let mut criterion = criterion::Criterion::default().configure_from_args();

    macro_rules! benchmarks {
        ($($module:ident,)+) => {
            $(benchmarks::$module::benches(&mut criterion);)+
        };
    }

    benchmarks! {
        insert,
        get,
    }

    criterion.final_summary();
}
