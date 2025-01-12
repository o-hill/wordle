SCRIPT_DIR="$(dirname ${0?})"
DATA_DIR="data"

pushd "${SCRIPT_DIR?}/../${DATA_DIR?}" 2>&1 1>/dev/null

for num in {1..24}; do
    GZ_FILE=$(printf "1-%05d-of-00024.gz" $num)
    http "http://storage.googleapis.com/books/ngrams/books/20200217/eng/${GZ_FILE?}" > "${GZ_FILE?}"
done

popd 2>&1 1>/dev/null
