use super::checks::CheckResult;

pub fn all_passed(checks: &[CheckResult]) -> bool {
    checks.iter().all(|check| check.ok)
}
