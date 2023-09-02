use std::path::PathBuf;

use clap::{Parser, Subcommand};
use color_eyre::eyre::{Context, ContextCompat, Result};
use gitlab::{AccessLevel, Gitlab};

mod create_repos;
mod models;
mod projects;

#[derive(Parser, Debug)]
struct Args {
    /// Gitlab host
    #[arg(long, default_value = "gitlab.ewi.tudelft.nl")]
    host: String,

    /// Gitlab API token
    #[arg(long = "token", env = "GITLAB_TOKEN", hide_env_values = true)]
    gitlab_token: String,

    /// Gitlab API user (connected to the token)
    #[arg(long = "user", env = "GITLAB_USER", hide_env_values = true)]
    gitlab_user: String,

    #[clap(subcommand)]
    cmd: SubCommand,
}

#[derive(Subcommand, Debug)]
enum SubCommand {
    /// Lists all projects in a given group with ssh clone url
    Projects {
        /// Gitlab group ID
        #[arg(required = true)]
        group_id: u64,
    },

    /// Unprotects a given branch on all projects in a given group
    Unprotect {
        /// Gitlab group ID
        #[arg(required = true)]
        group_id: u64,

        /// Branch to unprotect
        #[arg(required = true)]
        branch: String,
    },

    /// Create Gitlab repos for individual students under a certain group and
    CreateIndividualRepos {
        /// Parent Group ID
        #[arg(short, long, required = true)]
        group_id: u64,

        /// Template Repository to use for each student (has to be public)
        #[arg(short, long = "template", required = true)]
        template_repository: String,

        /// Brightspace student list (see README)
        #[arg(short, long, default_value = "./classlist.json", required = true)]
        student_list: PathBuf,

        /// Prefix to add to all created repositories
        #[arg(short = 'p', long = "prefix")]
        repo_name_prefix: Option<String>,

        /// Specify the accesslevel of the users to be added to the repo
        /// 
        /// Anonymous => 0,  
        /// Guest => 10,  
        /// Reporter => 20,  
        /// Developer => 30,  
        /// Maintainer => 40,
        /// Owner => 50,
        /// Admin => 60,
        #[arg(short,long, required = true, default_value_t = AccessLevel::Developer.into())]
        access_level: u64,
    },
}

fn main() -> Result<()> {
    color_eyre::install()?;
    dotenv::dotenv().ok();
    let args = Args::parse();

    let client =
        Gitlab::new(&args.host, &args.gitlab_token).wrap_err("failed to create gitlab client")?;

    match args.cmd {
        SubCommand::Projects { group_id } => projects::list(&client, group_id)?,
        SubCommand::Unprotect { group_id, branch } => {
            projects::unprotect(&client, group_id, &branch)?;
        }
        SubCommand::CreateIndividualRepos {
            group_id,
            mut template_repository,
            student_list,
            repo_name_prefix,
            access_level,
        } => {
            if template_repository.contains(&args.host) {
                let (proto, suff) = template_repository
                    .split_once("://")
                    .wrap_err("invalid template url")?;
                template_repository = format!(
                    "{proto}://{}:{}@{suff}",
                    &args.gitlab_user, &args.gitlab_token
                );
            }

            create_repos::create_individual_repos(
                &client,
                &repo_name_prefix,
                group_id,
                student_list,
                &template_repository,
                access_level.into(),
            )?;
        }
    }

    Ok(())
}
