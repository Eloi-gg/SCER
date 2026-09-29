git submodule update --remote --recursive &&
git pull &&
PARENT_DIR="$(dirname "$0")"
git add $PARENT_DIR/* &&
git commit -m "Update submodules to latest commit"
