#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd -- "${SCRIPT_DIR}/.." && pwd)"

TARGET_DIR_DEFAULT="${REPO_ROOT}/linkpad/resources/bin"
TARGET_DIR="${LINKPAD_BUNDLED_MIHOMO_DIR:-${TARGET_DIR_DEFAULT}}"
TARGET_OUT="${LINKPAD_BUNDLED_MIHOMO_OUT:-}"

RELEASE_LATEST_PAGE="https://github.com/MetaCubeX/mihomo/releases/latest"
RELEASE_API_LATEST="https://api.github.com/repos/MetaCubeX/mihomo/releases/latest"
RELEASE_API_TAG_PREFIX="https://api.github.com/repos/MetaCubeX/mihomo/releases/tags"
GITHUB_TOKEN="${LINKPAD_GITHUB_TOKEN:-}"

TEST_MODE="${LINKPAD_BUNDLED_MIHOMO_TEST_MODE:-0}"
TEST_DIR="${LINKPAD_BUNDLED_MIHOMO_TEST_DIR:-}"
STRICT_MODE="${LINKPAD_BUNDLED_MIHOMO_STRICT:-0}"
REFRESH_MODE="${LINKPAD_BUNDLED_MIHOMO_REFRESH:-}"

# --- normalize helpers -------------------------------------------------------

to_lower() {
  printf '%s' "${1:-}" | tr '[:upper:]' '[:lower:]'
}

normalize_os_tag() {
  case "$(to_lower "${1:-}")" in
    darwin | macos | mac | osx) echo "darwin" ;;
    windows | win32 | mingw | msys | cygwin) echo "windows" ;;
    linux) echo "linux" ;;
    android) echo "android" ;;
    *)
      echo "unsupported target os: ${1}" >&2
      return 1
      ;;
  esac
}

normalize_arch_tag() {
  case "$(to_lower "${1:-}")" in
    arm64 | aarch64) echo "arm64" ;;
    amd64 | x86_64) echo "amd64" ;;
    386 | x86 | i386 | i686) echo "386" ;;
    arm | armv7 | armv7l) echo "armv7" ;;
    *)
      echo "unsupported target arch: ${1}" >&2
      return 1
      ;;
  esac
}

normalize_tag() {
  local tag="${1:-}"
  tag="${tag#"${tag%%[![:space:]]*}"}"
  tag="${tag%"${tag##*[![:space:]]}"}"
  if [[ -z "${tag}" ]]; then
    return 1
  fi
  if [[ "${tag}" == v* ]]; then
    echo "${tag}"
  else
    echo "v${tag}"
  fi
}

# --- target resolution -------------------------------------------------------

target_from_triple() {
  local triple_lc="$(to_lower "${1:-}")"
  local key="${2:-}"
  case "${key}" in
    os)
      if [[ "${triple_lc}" == *"windows"* ]]; then
        echo "windows"
      elif [[ "${triple_lc}" == *"darwin"* || "${triple_lc}" == *"apple"* ]]; then
        echo "darwin"
      elif [[ "${triple_lc}" == *"linux"* ]]; then
        echo "linux"
      elif [[ "${triple_lc}" == *"android"* ]]; then
        echo "android"
      fi
      ;;
    arch)
      if [[ "${triple_lc}" == aarch64-* || "${triple_lc}" == arm64-* ]]; then
        echo "arm64"
      elif [[ "${triple_lc}" == x86_64-* || "${triple_lc}" == amd64-* ]]; then
        echo "amd64"
      elif [[ "${triple_lc}" == i686-* || "${triple_lc}" == i386-* || "${triple_lc}" == x86-* ]]; then
        echo "386"
      elif [[ "${triple_lc}" == armv7-* || "${triple_lc}" == arm-* ]]; then
        echo "armv7"
      fi
      ;;
    *) return 1 ;;
  esac
}

resolve_host_os() {
  case "$(uname -s)" in
    Darwin) normalize_os_tag "darwin" ;;
    Linux) normalize_os_tag "linux" ;;
    MINGW* | MSYS* | CYGWIN*) normalize_os_tag "windows" ;;
    *)
      echo "unsupported host os: $(uname -s)" >&2
      return 1
      ;;
  esac
}

resolve_host_arch() {
  case "$(uname -m)" in
    arm64 | aarch64) normalize_arch_tag "arm64" ;;
    x86_64 | amd64) normalize_arch_tag "amd64" ;;
    x86 | i386 | i686) normalize_arch_tag "386" ;;
    armv7 | armv7l) normalize_arch_tag "armv7" ;;
    *)
      echo "unsupported host arch: $(uname -m)" >&2
      return 1
      ;;
  esac
}

resolve_target_os() {
  if [[ -n "${LINKPAD_BUNDLED_MIHOMO_OS:-}" ]]; then
    normalize_os_tag "${LINKPAD_BUNDLED_MIHOMO_OS}"
    return 0
  fi

  local triple="${LINKPAD_BUNDLED_MIHOMO_TARGET:-${CARGO_BUILD_TARGET:-${TARGET:-}}}"
  if [[ -n "${triple}" ]]; then
    local from_triple=""
    from_triple="$(target_from_triple "${triple}" os || true)"
    if [[ -n "${from_triple}" ]]; then
      normalize_os_tag "${from_triple}"
      return 0
    fi
  fi

  resolve_host_os
}

resolve_target_arch() {
  if [[ -n "${LINKPAD_BUNDLED_MIHOMO_ARCH:-}" ]]; then
    normalize_arch_tag "${LINKPAD_BUNDLED_MIHOMO_ARCH}"
    return 0
  fi

  local triple="${LINKPAD_BUNDLED_MIHOMO_TARGET:-${CARGO_BUILD_TARGET:-${TARGET:-}}}"
  if [[ -n "${triple}" ]]; then
    local from_triple=""
    from_triple="$(target_from_triple "${triple}" arch || true)"
    if [[ -n "${from_triple}" ]]; then
      normalize_arch_tag "${from_triple}"
      return 0
    fi
  fi

  resolve_host_arch
}

os_tag="$(resolve_target_os)"
arch_tag="$(resolve_target_arch)"

if [[ "${os_tag}" == "windows" ]]; then
  binary_name="mihomo.exe"
  archive_ext=".zip"
else
  binary_name="mihomo"
  archive_ext=".gz"
fi

candidate_prefix="mihomo-${os_tag}-${arch_tag}-"

ensure_required_tools() {
  local required=(curl)
  if [[ "${archive_ext}" == ".zip" ]]; then
    required+=(unzip)
  else
    required+=(gzip)
  fi

  local cmd=""
  for cmd in "${required[@]}"; do
    if ! command -v "${cmd}" >/dev/null 2>&1; then
      echo "missing required command: ${cmd}" >&2
      exit 1
    fi
  done
}

ensure_required_tools
mkdir -p "${TARGET_DIR}"

existing_internal_binary() {
  if [[ -n "${TARGET_OUT}" ]]; then
    if [[ -f "${TARGET_OUT}" ]]; then
      echo "${TARGET_OUT}"
    fi
    return 0
  fi

  if [[ "${os_tag}" == "windows" ]]; then
    find "${TARGET_DIR}" -maxdepth 1 -type f -name "${candidate_prefix}*.exe" | sort | tail -n 1
  else
    find "${TARGET_DIR}" -maxdepth 1 -type f -name "${candidate_prefix}*" ! -name "*.zip" ! -name "*.gz" | sort | tail -n 1
  fi
}

apply_exec_permissions() {
  local path="${1}"
  chmod 755 "${path}" 2>/dev/null || true
}

current_internal="$(existing_internal_binary || true)"
if [[ -n "${current_internal}" && -z "${REFRESH_MODE}" ]]; then
  apply_exec_permissions "${current_internal}"
  echo "Bundled mihomo already exists at ${current_internal}"
  exit 0
fi

if [[ "${STRICT_MODE}" == "1" ]]; then
  if [[ -n "${current_internal}" ]]; then
    apply_exec_permissions "${current_internal}"
    echo "Bundled mihomo strict mode: using existing ${current_internal}"
    exit 0
  fi
  if [[ -n "${TARGET_OUT}" ]]; then
    echo "bundled mihomo missing at ${TARGET_OUT} and strict mode is enabled" >&2
  else
    echo "bundled mihomo missing under ${TARGET_DIR} (${candidate_prefix}*) and strict mode is enabled" >&2
  fi
  exit 1
fi

# --- release asset resolution ------------------------------------------------

build_asset_candidates() {
  local tag="${1:-}"
  local normalized_tag=""
  normalized_tag="$(normalize_tag "${tag}")" || return 1

  cat <<EOF2
${candidate_prefix}${normalized_tag}${archive_ext}
${candidate_prefix}go124-${normalized_tag}${archive_ext}
${candidate_prefix}go123-${normalized_tag}${archive_ext}
${candidate_prefix}go122-${normalized_tag}${archive_ext}
${candidate_prefix}go121-${normalized_tag}${archive_ext}
${candidate_prefix}go120-${normalized_tag}${archive_ext}
${candidate_prefix}v1-${normalized_tag}${archive_ext}
${candidate_prefix}v2-${normalized_tag}${archive_ext}
${candidate_prefix}v3-${normalized_tag}${archive_ext}
${candidate_prefix}v1-go124-${normalized_tag}${archive_ext}
${candidate_prefix}v1-go123-${normalized_tag}${archive_ext}
${candidate_prefix}v1-go122-${normalized_tag}${archive_ext}
${candidate_prefix}v1-go121-${normalized_tag}${archive_ext}
${candidate_prefix}v1-go120-${normalized_tag}${archive_ext}
${candidate_prefix}v2-go124-${normalized_tag}${archive_ext}
${candidate_prefix}v2-go123-${normalized_tag}${archive_ext}
${candidate_prefix}v2-go122-${normalized_tag}${archive_ext}
${candidate_prefix}v2-go121-${normalized_tag}${archive_ext}
${candidate_prefix}v2-go120-${normalized_tag}${archive_ext}
${candidate_prefix}v3-go124-${normalized_tag}${archive_ext}
${candidate_prefix}v3-go123-${normalized_tag}${archive_ext}
${candidate_prefix}v3-go122-${normalized_tag}${archive_ext}
${candidate_prefix}v3-go121-${normalized_tag}${archive_ext}
${candidate_prefix}v3-go120-${normalized_tag}${archive_ext}
EOF2

  if [[ "${arch_tag}" == "amd64" ]]; then
    echo "${candidate_prefix}compatible-${normalized_tag}${archive_ext}"
  fi
}

fetch_latest_tag_from_web() {
  local effective_url=""
  effective_url="$(curl -fsSL -o /dev/null -w '%{url_effective}' "${RELEASE_LATEST_PAGE}")" || return 1
  local marker="/releases/tag/"
  if [[ "${effective_url}" != *"${marker}"* ]]; then
    return 1
  fi
  local raw_tag="${effective_url#*${marker}}"
  raw_tag="${raw_tag%%\?*}"
  raw_tag="${raw_tag%%\#*}"
  raw_tag="${raw_tag%%/*}"
  normalize_tag "${raw_tag}"
}

release_json=""
archive_path="$(mktemp "${TMPDIR:-/tmp}/linkpad-mihomo.XXXXXX${archive_ext}")"
cleanup() {
  if [[ -n "${release_json}" ]]; then
    rm -f "${release_json}"
  fi
  if [[ -n "${archive_path}" ]]; then
    rm -f "${archive_path}"
  fi
}
trap cleanup EXIT

tag_name=""
asset_name=""

curl_release_api_to_json() {
  local url="${1}"
  release_json="$(mktemp)"
  local curl_args=(-fsSL)
  if [[ -n "${GITHUB_TOKEN}" ]]; then
    curl_args+=(-H "Authorization: Bearer ${GITHUB_TOKEN}")
  fi
  curl_args+=("${url}")
  curl "${curl_args[@]}" -o "${release_json}"
}

download_from_release_json() {
  if ! command -v jq >/dev/null 2>&1; then
    return 1
  fi

  local asset_line=""
  tag_name="$(jq -r '.tag_name // empty' "${release_json}")"
  asset_line="$(
    jq -r --arg prefix "${candidate_prefix}" --arg ext "${archive_ext}" '
      .assets
      | map(select(.name | startswith($prefix) and endswith($ext)))
      | sort_by([(.name | test("alpha"; "i")), (.name | test("compatible"; "i")), (.name | test("-go")), (.name | length)])
      | first
      | if . == null then "" else "\(.name)|\(.browser_download_url)" end
    ' "${release_json}"
  )"

  if [[ -z "${asset_line}" ]]; then
    return 1
  fi

  asset_name="${asset_line%%|*}"
  local asset_url="${asset_line#*|}"
  curl -fL "${asset_url}" -o "${archive_path}"
}

download_with_release_api_latest() {
  curl_release_api_to_json "${RELEASE_API_LATEST}" || return 1
  download_from_release_json
}

download_with_release_api_tag() {
  local requested_tag="${1:-}"
  requested_tag="$(normalize_tag "${requested_tag}")" || return 1
  curl_release_api_to_json "${RELEASE_API_TAG_PREFIX}/${requested_tag}" || return 1
  download_from_release_json
}

download_with_candidates() {
  local tag="${1:-}"
  tag="$(normalize_tag "${tag}")" || return 1

  local candidate=""
  while IFS= read -r candidate; do
    [[ -z "${candidate}" ]] && continue
    local by_tag="https://github.com/MetaCubeX/mihomo/releases/download/${tag}/${candidate}"
    local latest="https://github.com/MetaCubeX/mihomo/releases/latest/download/${candidate}"
    if curl -fL "${by_tag}" -o "${archive_path}" || curl -fL "${latest}" -o "${archive_path}"; then
      tag_name="${tag}"
      asset_name="${candidate}"
      return 0
    fi
  done < <(build_asset_candidates "${tag}")

  return 1
}

# --- install helpers ---------------------------------------------------------

target_path_from_name() {
  local name="${1:-}"
  local path=""
  if [[ -n "${TARGET_OUT}" ]]; then
    path="${TARGET_OUT}"
  else
    path="${TARGET_DIR}/${name}"
  fi
  mkdir -p "$(dirname -- "${path}")"
  echo "${path}"
}

target_path_from_asset_name() {
  local archive_name="${1:-}"
  local base="${archive_name}"
  base="${base%${archive_ext}}"
  local out_name="${base}"
  if [[ "${os_tag}" == "windows" && "${out_name}" != *.exe ]]; then
    out_name="${out_name}.exe"
  fi
  target_path_from_name "${out_name}"
}

extract_archive_to_target() {
  local src_archive="${1}"
  local dst_path="${2}"

  if [[ "${archive_ext}" == ".zip" ]]; then
    local unzip_dir=""
    unzip_dir="$(mktemp -d "${TMPDIR:-/tmp}/linkpad-mihomo-unzip.XXXXXX")"
    unzip -oq "${src_archive}" -d "${unzip_dir}"

    local extracted=""
    extracted="$(find "${unzip_dir}" -type f \( -iname "mihomo.exe" -o -iname "mihomo*.exe" \) | head -n 1 || true)"
    if [[ -z "${extracted}" || ! -f "${extracted}" ]]; then
      rm -rf "${unzip_dir}"
      echo "failed to locate mihomo.exe in downloaded archive ${asset_name}" >&2
      return 1
    fi
    cp "${extracted}" "${dst_path}"
    rm -rf "${unzip_dir}"
    return 0
  fi

  gzip -dc "${src_archive}" > "${dst_path}"
}

cleanup_stale_targets() {
  local current="${1}"
  if [[ -n "${TARGET_OUT}" ]]; then
    return 0
  fi

  local path=""
  while IFS= read -r path; do
    [[ -z "${path}" ]] && continue
    if [[ "${path}" != "${current}" ]]; then
      rm -f "${path}"
    fi
  done < <(find "${TARGET_DIR}" -maxdepth 1 -type f -name "${candidate_prefix}*" ! -name "*.zip" ! -name "*.gz")

  for legacy in "${TARGET_DIR}/mihomo" "${TARGET_DIR}/mihomo.exe"; do
    if [[ -f "${legacy}" && "${legacy}" != "${current}" ]]; then
      rm -f "${legacy}"
    fi
  done
}

install_local_binary() {
  local src_binary="${1}"
  local src_name="$(basename -- "${src_binary}")"

  if [[ "${src_name}" == "mihomo" || "${src_name}" == "mihomo.exe" ]]; then
    if [[ "${os_tag}" == "windows" ]]; then
      src_name="${candidate_prefix}local-test.exe"
    else
      src_name="${candidate_prefix}local-test"
    fi
  fi

  local dst_path=""
  dst_path="$(target_path_from_name "${src_name}")"
  cp "${src_binary}" "${dst_path}"
  apply_exec_permissions "${dst_path}"
  cleanup_stale_targets "${dst_path}"

  echo "Bundled mihomo prepared from local test binary: ${src_binary} -> ${dst_path} [${os_tag}/${arch_tag}]"
}

install_local_archive() {
  local src_archive="${1}"
  local archive_name="$(basename -- "${src_archive}")"
  asset_name="${archive_name}"

  local dst_path=""
  dst_path="$(target_path_from_asset_name "${archive_name}")"
  extract_archive_to_target "${src_archive}" "${dst_path}"
  apply_exec_permissions "${dst_path}"
  cleanup_stale_targets "${dst_path}"

  echo "Bundled mihomo prepared from local test archive: ${src_archive} -> ${dst_path} [${os_tag}/${arch_tag}]"
}

prepare_from_test_dir() {
  if [[ -z "${TEST_DIR}" ]]; then
    echo "test mode requires LINKPAD_BUNDLED_MIHOMO_TEST_DIR" >&2
    exit 1
  fi
  if [[ ! -d "${TEST_DIR}" ]]; then
    echo "test dir does not exist: ${TEST_DIR}" >&2
    exit 1
  fi

  local local_archive=""
  local_archive="$(find "${TEST_DIR}" -maxdepth 1 -type f -name "${candidate_prefix}*${archive_ext}" | sort | tail -n 1 || true)"
  if [[ -n "${local_archive}" ]]; then
    install_local_archive "${local_archive}"
    return 0
  fi

  local local_binary=""
  if [[ "${os_tag}" == "windows" ]]; then
    local_binary="$(find "${TEST_DIR}" -maxdepth 1 -type f \( -name "${candidate_prefix}*.exe" -o -name "mihomo.exe" \) | sort | tail -n 1 || true)"
  else
    local_binary="$(find "${TEST_DIR}" -maxdepth 1 -type f \( -name "${candidate_prefix}*" -o -name "mihomo" \) ! -name "*.zip" ! -name "*.gz" | sort | tail -n 1 || true)"
  fi

  if [[ -n "${local_binary}" ]]; then
    install_local_binary "${local_binary}"
    return 0
  fi

  echo "test mode could not find matching mihomo in ${TEST_DIR} (expected prefix: ${candidate_prefix})" >&2
  exit 1
}

if [[ "${TEST_MODE}" == "1" ]]; then
  prepare_from_test_dir
  exit 0
fi

# --- online fetch path -------------------------------------------------------

requested_tag="$(normalize_tag "${LINKPAD_BUNDLED_MIHOMO_VERSION:-}" || true)"
if [[ -n "${requested_tag}" ]]; then
  if ! download_with_release_api_tag "${requested_tag}" && ! download_with_candidates "${requested_tag}"; then
    echo "failed to download mihomo ${requested_tag} for ${os_tag}/${arch_tag}" >&2
    exit 1
  fi
else
  if ! download_with_release_api_latest; then
    latest_tag="$(fetch_latest_tag_from_web || true)"
    if [[ -z "${latest_tag}" ]] || ! download_with_candidates "${latest_tag}"; then
      echo "failed to download latest mihomo for ${os_tag}/${arch_tag}" >&2
      exit 1
    fi
  fi
fi

dst_path="$(target_path_from_asset_name "${asset_name}")"
extract_archive_to_target "${archive_path}" "${dst_path}"
apply_exec_permissions "${dst_path}"
cleanup_stale_targets "${dst_path}"

echo "Bundled mihomo downloaded (${tag_name:-latest}, ${asset_name}) -> ${dst_path} [${os_tag}/${arch_tag}]"
