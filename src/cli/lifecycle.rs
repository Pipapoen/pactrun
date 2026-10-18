use super::*;
use crate::domain::{LocalSelector, ObjectDeletion, ObjectDeletionResult, PackageId};

pub(super) enum PackageReference {
    Exact(PackageId),
    Named(LocalSelector),
}

pub(super) fn parse_package_delete(parser: &mut Parser) -> Result<LifecycleCommand, CliError> {
    let value = required_value_string(parser, "Package reference")?;
    let reference = match value.parse() {
        Ok(id) => PackageReference::Exact(id),
        Err(_) => PackageReference::Named(
            LocalSelector::parse(value).map_err(|e| CliError::usage(e.to_string()))?,
        ),
    };
    require_end(parser)?;
    Ok(LifecycleCommand::Package(reference))
}

pub(super) enum LifecycleCommand {
    Package(PackageReference),
    Collect {
        plan: bool,
    },
    Revision(RevisionReference),
    Run {
        run: Selector<RunId>,
        delete_artifacts: bool,
    },
}
impl LifecycleCommand {
    pub(super) fn resolve_ids(&mut self, r: &mut short_ids::Resolver<'_>) -> Result<(), CliError> {
        match self {
            Self::Package(reference) => r.package(reference),
            Self::Run { run, .. } => run.resolve(r),
            Self::Revision(v) => r.revision(v),
            _ => Ok(()),
        }
    }
}

pub(super) fn parse_run_delete(parser: &mut Parser) -> Result<LifecycleCommand, CliError> {
    let run = parse_run_id(&value_string(parser, "RunId")?)?;
    let mut delete_artifacts = false;
    while let Some(arg) = parser.next().map_err(lex_error)? {
        match arg {
            Arg::Long("delete-artifacts") if !delete_artifacts => delete_artifacts = true,
            _ => {
                return Err(CliError::usage(
                    "unsupported or duplicate Run deletion option",
                ));
            }
        }
    }
    Ok(LifecycleCommand::Run {
        run,
        delete_artifacts,
    })
}

pub(super) fn execute(
    command: LifecycleCommand,
    root: &Path,
    format: reply::OutputContext,
) -> Result<(), CliError> {
    if let LifecycleCommand::Collect { plan } = command {
        let app = if plan {
            PactrunApplication::open_read_only(root)
        } else {
            PactrunApplication::open_for_collection(root)
        }
        .map_err(app_error)?;
        let report = app.collect_content(!plan).map_err(app_error)?;
        let result = presentation::Collection::new(plan, &report);
        if report.failed != 0 {
            let mut error = CliError::operation(
                "collection incomplete; some removals could not be confirmed and may already be absent; retry after resolving the storage problem",
            );
            error.partial = Some(presentation::PartialResult::Collection(Box::new(result)));
            return Err(error);
        }
        return presentation::emit_result(format, "storage gc", &result);
    }
    let app = PactrunApplication::open(root).map_err(app_error)?;
    let target = match command {
        LifecycleCommand::Package(PackageReference::Exact(id)) => ObjectDeletion::Package(id),
        LifecycleCommand::Package(PackageReference::Named(_)) => {
            unreachable!("Package resolution precedes execution")
        }
        LifecycleCommand::Collect { .. } => unreachable!(),
        LifecycleCommand::Run {
            run,
            delete_artifacts,
        } => ObjectDeletion::Run {
            run: run.full(),
            delete_artifacts,
        },
        LifecycleCommand::Revision(reference) => ObjectDeletion::Revision(match reference {
            RevisionReference::Exact(identity) => identity,
            other => resolve_revision(&app, other)?,
        }),
    };
    let name = match target {
        ObjectDeletion::Package(_) => "package delete",
        ObjectDeletion::Run { .. } => "run delete",
        ObjectDeletion::Revision(_) => "revision delete",
        _ => unreachable!(),
    };
    render_deletion(format, name, app.delete_object(&target).map_err(app_error)?)
}

pub(super) fn render_deletion(
    format: reply::OutputContext,
    command: &str,
    result: ObjectDeletionResult,
) -> Result<(), CliError> {
    let outcome = match result {
        ObjectDeletionResult::Deleted => "deleted",
        ObjectDeletionResult::AlreadyAbsent => "already_absent",
        ObjectDeletionResult::Blocked(reason) => return Err(CliError::operation(reason.message())),
    };
    presentation::emit_result(format, command, &presentation::Deletion { outcome })
}
