use super::Context;
use crate::argv::EnvOptions;

pub(crate) struct Env {
    argv: EnvOptions,
}

impl From<EnvOptions> for Env {
    fn from(argv: EnvOptions) -> Self {
        Env { argv }
    }
}

// pub fn run(ctx: crate::Context) -> Result<Context, super::Error> {
//     let session = env::var(KAKOUNE_SESSION)?;
//     let client = env::var(KAKOUNE_CLIENT).ok();
//     Ok(Context { session, client })
// }

impl Env {
    pub fn run(&self, ctx: Context) {
        println!("{:?}", self.argv);
        println!("{:?}", ctx);
    }
}
