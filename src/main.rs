use std::{fs::File, path::PathBuf};

use clap::{Args, Parser, Subcommand};
use color_eyre::eyre::{Context, ContextCompat, Result};
use git::create_repos;
use gitlab::{api::common::AccessLevel, Gitlab};

use crate::git::projects;

mod brightspace;
mod git;
mod models;

#[derive(Debug, Parser)]
struct Cli {
    #[arg(long, default_value_t = false)]
    dry_run: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Lists all projects in a given group with ssh clone url
    Projects {
        /// The group id to list the projects for
        #[arg(required = true)]
        group_id: u64,

        #[command(flatten)]
        gitlab: GitlabArgs,
    },

    Unprotect {
        /// The group id to unprotect the branches for
        #[arg(required = true)]
        group_id: u64,

        #[arg(default_value = "main")]
        branch: String,

        #[command(flatten)]
        gitlab: GitlabArgs,
    },

    Unfork {
        #[arg(required = true)]
        group_id: u64,

        #[command(flatten)]
        gitlab: GitlabArgs,
    },

    CreateIndividualRepos {
        /// Brightspace Organizational Unit ID to use the classlist from
        #[arg(long = "ou", required = true)]
        brightspace_ou: u64,

        #[command(flatten)]
        brightspace: BrightspaceArgs,

        #[command(flatten)]
        gitlab: GitlabArgs,

        #[command(flatten)]
        project: GitlabProjectCreationArgs,

        /// Prefix to add to all created repositories
        #[arg(short = 'p', long = "prefix")]
        repo_name_prefix: Option<String>,
    },

    CreateGroupReposBrightspace {
        #[arg(short, long = "brightspace", required = true)]
        brightspace_group_id: u64,

        #[command(flatten)]
        gitlab: GitlabArgs,

        #[command(flatten)]
        brightspace: BrightspaceArgs,

        #[command(flatten)]
        project: GitlabProjectCreationArgs,
    },

    ClasslistCsv {
        #[arg(required = true)]
        course_id: u64,

        #[arg(short, long = "file", default_value = "classlist.csv")]
        output_file: PathBuf,

        #[arg(long, default_value_t = false)]
        gitbull: bool,

        #[command(flatten)]
        brightspace: BrightspaceArgs,
    },
}

#[derive(Debug, Args)]
struct GitlabArgs {
    /// Gitlab host
    #[arg(long, hide = true, default_value = "gitlab.ewi.tudelft.nl")]
    host: String,

    /// Gitlab API user (connected to the token)
    #[arg(long, env = "GITLAB_USER", hide_env_values = true)]
    user: String,

    /// Gitlab API token
    #[arg(long, env = "GITLAB_TOKEN", hide_env_values = true)]
    token: String,
}

#[derive(Debug, Args)]
struct BrightspaceArgs {
    #[arg(long, hide = true, default_value = "https://brightspace.tudelft.nl")]
    base_url: http::Uri,

    /// Brightspace Cookie
    #[arg(long, env = "BRIGHTSPACE_COOKIE", hide_env_values = true)]
    cookie: String,

    // Brightspace LTI Session ID
    #[arg(long, env = "BRIGHTSPACE_SESSIONID", hide_env_values = true)]
    session_id: String,
}

#[derive(Debug, Args)]
struct GitlabProjectCreationArgs {
    /// Gitlab Group ID under which to create the repositories
    #[arg(required = true)]
    gitlab_group_id: u64,

    /// Template repository to initialize student repos with
    #[arg(required = true, short, long = "template")]
    template_repository: String,

    /// Specify the accesslevel of the users to be added to the repo
    ///
    /// Anonymous => 0,  
    /// Guest => 10,  
    /// Reporter => 20,  
    /// Developer => 30,  
    /// Maintainer => 40,
    /// Owner => 50,
    /// Admin => 60,
    #[arg(short, long, default_value_t = AccessLevel::Developer.as_u64())]
    access_level: u64,
}

fn authenticate_template_repo_url(
    mut template_repository: String,
    host: &str,
    user: &str,
    token: &str,
) -> Result<String> {
    if template_repository.contains(host) {
        let (proto, suff) = template_repository
            .split_once("://")
            .wrap_err("invalid template url")?;
        template_repository = format!("{proto}://{user}:{token}@{suff}");
    }

    Ok(template_repository)
}

fn u64_to_access_level(access: u64) -> AccessLevel {
    if access >= 60 {
        AccessLevel::Admin
    } else if access >= 50 {
        AccessLevel::Owner
    } else if access >= 40 {
        AccessLevel::Maintainer
    } else if access >= 30 {
        AccessLevel::Developer
    } else if access >= 20 {
        AccessLevel::Reporter
    } else if access >= 10 {
        AccessLevel::Guest
    } else {
        AccessLevel::Anonymous
    }
}

fn main() -> Result<()> {
    color_eyre::install()?;
    dotenv::dotenv().ok();
    let cli = Cli::parse();

    // let client =
    match cli.command {
        Commands::Projects { gitlab, group_id } => {
            let client =
                Gitlab::new(&gitlab.host, &gitlab.token).wrap_err("failed to create git client")?;

            projects::list(&client, group_id)?;
        }
        Commands::Unprotect {
            gitlab,
            group_id,
            branch,
        } => {
            let client =
                Gitlab::new(&gitlab.host, &gitlab.token).wrap_err("failed to create git client")?;

            projects::unprotect(&client, group_id, &branch, cli.dry_run)?;
        }
        Commands::Unfork { group_id, gitlab } => {
            let client =
                Gitlab::new(&gitlab.host, &gitlab.token).wrap_err("failed to create git client")?;
            // projects::unfork(&client, group_id, cli.dry_run)?;
        }
        Commands::ClasslistCsv {
            course_id,
            output_file,
            brightspace,
            gitbull,
        } => {
            let f = File::create(output_file).wrap_err("could not create output file")?;
            let mut wtr = csv::Writer::from_writer(f);

            let out =
                brightspace::get_students(&brightspace.base_url, &brightspace.cookie, course_id)?;

            if gitbull {
                out.iter()
                    .try_for_each(|el| wtr.write_record(&[&el.netid, &el.email, &el.netid]))?;
            } else {
                out.iter().try_for_each(|el| wtr.serialize(el))?;
            }
            wtr.flush()?;
        }
        Commands::CreateIndividualRepos {
            gitlab,
            project,
            repo_name_prefix,
            brightspace,
            brightspace_ou,
        } => {
            let client =
                Gitlab::new(&gitlab.host, &gitlab.token).wrap_err("failed to create git client")?;

            let template = authenticate_template_repo_url(
                project.template_repository,
                &gitlab.host,
                &gitlab.user,
                &gitlab.token,
            )?;

            create_repos::create_individual_repos(
                &client,
                &repo_name_prefix,
                project.gitlab_group_id,
                &template,
                u64_to_access_level(project.access_level),
                &brightspace.cookie,
                &brightspace.base_url,
                brightspace_ou,
                cli.dry_run,
            )?;
        }
        Commands::CreateGroupReposBrightspace {
            brightspace_group_id,
            gitlab,
            brightspace,
            project,
        } => {
            let client =
                Gitlab::new(&gitlab.host, &gitlab.token).wrap_err("failed to create git client")?;

            let template = authenticate_template_repo_url(
                project.template_repository,
                &gitlab.host,
                &gitlab.user,
                &gitlab.token,
            )?;

            let groups = brightspace::get_groups(
                &brightspace.session_id,
                &brightspace_group_id.to_string(),
            )?;

            create_repos::create_group_repos(
                &client,
                project.gitlab_group_id,
                &template,
                u64_to_access_level(project.access_level),
                &groups,
                cli.dry_run,
            )?;
        }
    }

    Ok(())
}
