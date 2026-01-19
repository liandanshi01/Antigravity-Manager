#!/usr/bin/env bash
set -euo pipefail

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# 获取项目根目录
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# 显示用法
usage() {
    cat <<EOF
用法: $0 <new-version> [options]

参数:
  new-version      新版本号 (例如: 3.3.34 或 v3.3.34)

选项:
  --no-commit      不自动创建 git commit
  --no-tag         不自动创建 git tag
  --dry-run        仅显示将要修改的内容，不实际修改
  -h, --help       显示此帮助信息

示例:
  $0 3.3.35                    # 更新版本号并创建 commit 和 tag
  $0 3.3.35 --no-commit        # 仅更新文件，不提交
  $0 3.3.35 --dry-run          # 预览修改
EOF
    exit 1
}

# 参数解析
NEW_VERSION=""
DO_COMMIT=true
DO_TAG=true
DRY_RUN=false

while [[ $# -gt 0 ]]; do
    case $1 in
        --no-commit)
            DO_COMMIT=false
            shift
            ;;
        --no-tag)
            DO_TAG=false
            shift
            ;;
        --dry-run)
            DRY_RUN=true
            shift
            ;;
        -h|--help)
            usage
            ;;
        *)
            if [[ -z "$NEW_VERSION" ]]; then
                NEW_VERSION="$1"
            else
                echo -e "${RED}错误: 未知参数 '$1'${NC}"
                usage
            fi
            shift
            ;;
    esac
done

# 检查版本号参数
if [[ -z "$NEW_VERSION" ]]; then
    echo -e "${RED}错误: 请指定新版本号${NC}"
    usage
fi

# 去掉 v 前缀（如果有）
NEW_VERSION="${NEW_VERSION#v}"

# 验证版本号格式
if ! [[ "$NEW_VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
    echo -e "${RED}错误: 版本号格式不正确，应为 x.y.z 格式${NC}"
    exit 1
fi

cd "$PROJECT_ROOT"

# 获取当前版本
CURRENT_VERSION=$(grep -m1 '"version"' package.json | sed 's/.*"version": "\(.*\)".*/\1/')

echo -e "${YELLOW}当前版本: ${CURRENT_VERSION}${NC}"
echo -e "${GREEN}新版本: ${NEW_VERSION}${NC}"
echo ""

# 定义要修改的文件
declare -A FILES=(
    ["package.json"]="s/\"version\": \".*\"/\"version\": \"${NEW_VERSION}\"/"
    ["src-tauri/Cargo.toml"]="s/^version = \".*\"/version = \"${NEW_VERSION}\"/"
    ["src-tauri/tauri.conf.json"]="s/\"version\": \".*\"/\"version\": \"${NEW_VERSION}\"/"
)

# 执行或显示修改
if [[ "$DRY_RUN" == true ]]; then
    echo -e "${YELLOW}[DRY RUN] 将要修改以下文件:${NC}"
    for file in "${!FILES[@]}"; do
        echo "  - $file"
        sed -n "${FILES[$file]}p" "$file" 2>/dev/null || echo "    (未找到匹配)"
    done
else
    echo "更新文件中..."
    for file in "${!FILES[@]}"; do
        if [[ ! -f "$file" ]]; then
            echo -e "${RED}错误: 文件不存在: $file${NC}"
            exit 1
        fi
        
        # 使用 sed 进行替换（macOS 兼容）
        if [[ "$OSTYPE" == "darwin"* ]]; then
            sed -i '' "${FILES[$file]}" "$file"
        else
            sed -i "${FILES[$file]}" "$file"
        fi
        
        echo -e "  ${GREEN}✓${NC} $file"
    done
    
    echo ""
    echo -e "${GREEN}版本号已更新至 ${NEW_VERSION}${NC}"
    
    # 同步 package-lock.json
    echo ""
    echo "同步 package-lock.json..."
    npm install --package-lock-only --silent
    echo -e "  ${GREEN}✓${NC} package-lock.json"
fi

# Git 操作
if [[ "$DRY_RUN" == false ]]; then
    if [[ "$DO_COMMIT" == true ]]; then
        echo ""
        echo "创建 Git commit..."
        git add package.json package-lock.json src-tauri/Cargo.toml src-tauri/tauri.conf.json
        git commit -m "chore: bump version to ${NEW_VERSION}"
        echo -e "${GREEN}✓ Commit 已创建${NC}"
        
        if [[ "$DO_TAG" == true ]]; then
            echo ""
            echo "创建 Git tag..."
            
            # 检查 tag 是否已存在
            if git rev-parse "v${NEW_VERSION}" >/dev/null 2>&1; then
                echo -e "${YELLOW}警告: Tag v${NEW_VERSION} 已存在${NC}"
                read -p "是否删除并重新创建? (y/N) " -n 1 -r
                echo
                if [[ $REPLY =~ ^[Yy]$ ]]; then
                    git tag -d "v${NEW_VERSION}"
                    git tag "v${NEW_VERSION}"
                    echo -e "${GREEN}✓ Tag 已重新创建${NC}"
                fi
            else
                git tag "v${NEW_VERSION}"
                echo -e "${GREEN}✓ Tag v${NEW_VERSION} 已创建${NC}"
            fi
            
            echo ""
            echo -e "${YELLOW}提示: 使用以下命令推送到远程:${NC}"
            echo "  git push origin feature/mut-key"
            echo "  git push origin v${NEW_VERSION}"
        fi
    fi
fi

echo ""
echo -e "${GREEN}完成！${NC}"
