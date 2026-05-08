// ============================================================================
// stick - 설정 관리 모듈
// 설정 파일 경로: ~/.config/stick/config.json
// ============================================================================

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

// ── 기본값 상수 정의 ──────────────────────────────────────────────────────
// "매직 넘버" 방지: 모든 기본값을 명확한 상수로 정의합니다.

/// 기본 제외 확장자 목록 (임시/시스템 파일)
const DEFAULT_EXCLUDE_EXTENSIONS: &[&str] = &[
    ".tmp", ".swp", ".swo", ".bak", ".crdownload", ".part", ".download",
];

/// 기본 제외 디렉토리 목록 (버전관리/IDE/빌드 폴더)
const DEFAULT_EXCLUDE_DIRECTORIES: &[&str] = &[
    ".git",
    "node_modules",
    ".idea",
    ".vscode",
    "__pycache__",
    ".venv",
    "target",
    ".DS_Store",
];

/// 기본 감시 간격 (초) - 이벤트 기반이므로 폴백용
const DEFAULT_SCAN_INTERVAL_SECONDS: u64 = 5;

// ── 설정 구조체 ──────────────────────────────────────────────────────────

/// stick의 전체 설정을 담는 구조체
/// ~/.config/stick/config.json에 직렬화되어 저장됩니다.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StickConfig {
    /// 감시할 폴더 경로 목록
    pub watch_paths: Vec<String>,

    /// 하위 디렉토리 재귀 탐색 여부
    pub recursive: bool,

    /// 숨김 파일(. 으로 시작하는 파일) 제외 여부
    pub exclude_hidden: bool,

    /// 심볼릭 링크 제외 여부
    pub exclude_symlinks: bool,

    /// 임시 파일 제외 여부
    pub exclude_temp_files: bool,

    /// 제외할 파일 확장자 목록
    pub exclude_extensions: Vec<String>,

    /// 제외할 디렉토리 이름 목록
    pub exclude_directories: Vec<String>,

    /// 로그 파일 저장 경로
    pub log_path: String,

    /// 스캔 간격 (초) - 폴링 모드에서 사용
    pub scan_interval_seconds: u64,

    /// 스캔 실행 시 대화형 확인(Y/N) 여부
    pub confirm_before_scan: bool,

    /// 로그 레벨 (info, debug 등)
    pub log_level: String,

    /// macOS 시스템 알림 활성화 여부 (기본값: false)
    pub enable_notifications: bool,

    /// 실시간 파일 변경 감지 후 변환 트리거 대기 시간 (초, 기본값: 2)
    pub debounce_delay_seconds: u64,

    /// 부팅 시 자동으로 백그라운드 데몬 실행 여부 (기본값: true)
    pub auto_start: bool,
}

impl Default for StickConfig {
    /// 안전한 기본 설정값 반환
    /// 사용자의 홈 디렉토리 기반으로 경로를 자동 설정합니다.
    fn default() -> Self {
        // 홈 디렉토리 기반 기본 로그 경로 계산
        let default_log_path = dirs::home_dir()
            .map(|home| home.join("logs").join("stick").to_string_lossy().to_string())
            .unwrap_or_else(|| "/tmp/stick/logs".to_string());

        Self {
            watch_paths: Vec::new(),
            recursive: true,
            exclude_hidden: true,
            exclude_symlinks: true,
            exclude_temp_files: true,
            exclude_extensions: DEFAULT_EXCLUDE_EXTENSIONS
                .iter()
                .map(|s| s.to_string())
                .collect(),
            exclude_directories: DEFAULT_EXCLUDE_DIRECTORIES
                .iter()
                .map(|s| s.to_string())
                .collect(),
            log_path: default_log_path,
            scan_interval_seconds: DEFAULT_SCAN_INTERVAL_SECONDS,
            confirm_before_scan: true,
            log_level: "info".to_string(),
            enable_notifications: false,
            debounce_delay_seconds: 2,
            auto_start: true,
        }
    }
}

impl StickConfig {
    /// 설정 파일 경로 반환
    /// ~/.config/stick/config.json
    pub fn config_file_path() -> Result<PathBuf> {
        let config_dir = dirs::home_dir()
            .context("홈 디렉토리를 찾을 수 없습니다")?
            .join(".config")
            .join("stick");

        Ok(config_dir.join("config.json"))
    }

    /// PID 파일 경로 반환 (향후 확장용)
    /// ~/.config/stick/stick.pid
    #[allow(dead_code)]
    pub fn pid_file_path() -> Result<PathBuf> {
        let config_dir = dirs::home_dir()
            .context("홈 디렉토리를 찾을 수 없습니다")?
            .join(".config")
            .join("stick");

        Ok(config_dir.join("stick.pid"))
    }

    /// 설정 파일에서 로드
    /// 파일이 없으면 기본값으로 새로 생성합니다.
    pub fn load() -> Result<Self> {
        let config_path = Self::config_file_path()?;

        if config_path.exists() {
            let content = fs::read_to_string(&config_path)
                .with_context(|| format!("설정 파일 읽기 실패: {:?}", config_path))?;

            let config: StickConfig = serde_json::from_str(&content)
                .with_context(|| "설정 파일 파싱 실패. JSON 형식을 확인해주세요.")?;

            Ok(config)
        } else {
            // 설정 파일이 없으면 기본값으로 생성
            let config = Self::default();
            config.save()?;
            Ok(config)
        }
    }

    /// 설정을 파일에 저장
    /// 디렉토리가 없으면 자동 생성합니다.
    pub fn save(&self) -> Result<()> {
        let config_path = Self::config_file_path()?;

        // 설정 디렉토리가 없으면 생성
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("설정 디렉토리 생성 실패: {:?}", parent))?;
        }

        // 보기 좋게 pretty-print JSON으로 저장
        let json = serde_json::to_string_pretty(self)
            .context("설정 직렬화 실패")?;

        fs::write(&config_path, json)
            .with_context(|| format!("설정 파일 저장 실패: {:?}", config_path))?;

        Ok(())
    }

    /// 설정값 유효성 검증
    /// 경로 존재 여부, 쓰기 권한 등을 확인합니다.
    pub fn validate(&self) -> Vec<String> {
        let mut warnings = Vec::new();

        // 감시 경로가 비어있으면 경고
        if self.watch_paths.is_empty() {
            warnings.push("감시할 폴더가 설정되지 않았습니다. 'stick config'로 추가해주세요.".to_string());
        }

        // 각 감시 경로의 존재 여부 확인
        for watch_path in &self.watch_paths {
            let path = Path::new(watch_path);
            if !path.exists() {
                warnings.push(format!("경로가 존재하지 않습니다: {}", watch_path));
            } else if !path.is_dir() {
                warnings.push(format!("디렉토리가 아닙니다: {}", watch_path));
            }
        }

        // 로그 경로의 부모 디렉토리 확인
        let log_path = Path::new(&self.log_path);
        if let Some(parent) = log_path.parent() {
            if !parent.exists() {
                warnings.push(format!(
                    "로그 경로의 상위 디렉토리가 존재하지 않습니다: {}",
                    parent.display()
                ));
            }
        }

        warnings
    }

    /// 로그 디렉토리 경로를 PathBuf로 반환
    pub fn log_dir(&self) -> PathBuf {
        // ~ 틸드를 실제 홈 경로로 확장
        if self.log_path.starts_with('~') {
            if let Some(home) = dirs::home_dir() {
                return home.join(&self.log_path[2..]);
            }
        }
        PathBuf::from(&self.log_path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = StickConfig::default();
        assert!(config.recursive);
        assert!(config.exclude_hidden);
        assert!(config.exclude_symlinks);
        assert!(config.watch_paths.is_empty());
        assert!(!config.exclude_extensions.is_empty());
    }

    #[test]
    fn test_serialization_roundtrip() {
        let config = StickConfig::default();
        let json = serde_json::to_string_pretty(&config).unwrap();
        let deserialized: StickConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(config.recursive, deserialized.recursive);
        assert_eq!(config.exclude_hidden, deserialized.exclude_hidden);
    }
}
