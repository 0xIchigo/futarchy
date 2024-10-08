#!/bin/sh

test() {
    find programs tests sdk | entr -sc '(cd sdk && yarn build) && RUST_LOG= anchor test'
}

test_no_build() {
    find programs tests sdk | entr -sc '(cd sdk && yarn build) && RUST_LOG= anchor test --skip-build'
}

# ./test.sh build_verifiable autocrat
build_verifiable() {
    PROGRAM_NAME=$1
    solana-verify build --library-name "$PROGRAM_NAME" -b ellipsislabs/solana:1.16.10
}

deploy() {
    PROGRAM_NAME=$1
    CLUSTER=$2
    solana program deploy --use-rpc -u "$CLUSTER" --program-id ./target/deploy/"$PROGRAM_NAME"-keypair.json ./target/deploy/"$PROGRAM_NAME".so --with-compute-unit-price 5 --max-sign-attempts 15 && PROGRAM_ID=$(solana-keygen pubkey ./target/deploy/"$PROGRAM_NAME"-keypair.json) && anchor idl init --filepath ./target/idl/"$PROGRAM_NAME".json $PROGRAM_ID --provider.cluster "$CLUSTER"
}

upgrade() {
    PROGRAM_NAME=$1
    PROGRAM_ID=$2
    CLUSTER=$3
    anchor upgrade ./target/deploy/"$PROGRAM_NAME".so -p "$PROGRAM_ID" --provider.cluster "$CLUSTER"
}

upgrade_idl() {
    PROGRAM_NAME=$1
    PROGRAM_ID=$2
    CLUSTER=$3
    anchor idl upgrade --filepath ./target/idl/"$PROGRAM_NAME".json "$PROGRAM_ID" --provider.cluster "$CLUSTER"
}

bankrun() {
    (find programs && find tests) | entr -csr 'anchor build -p autocrat && RUST_LOG= yarn run ts-mocha -p ./tsconfig.json -t 1000000 tests/autocrat.ts'
}

test_amm() {
    find programs tests | entr -csr 'anchor build -p amm && RUST_LOG= yarn run ts-mocha -p ./tsconfig.json -t 1000000 tests/amm.ts'
}

test_amm_logs() {
    find programs tests | entr -csr 'anchor build -p amm && yarn run ts-mocha -p ./tsconfig.json -t 1000000 tests/amm.ts'
}

# requires solana-test-validator to be running in the background
bankrun_vault() {
    (find programs && find tests) | entr -csr 'anchor build -p conditional_vault && RUST_LOG= yarn run ts-mocha -p ./tsconfig.json -t 1000000 tests/conditionalVault.ts'
}

bankrun_migrator() {
    (find programs && find tests) | entr -csr 'anchor build -p autocrat_migrator && RUST_LOG= yarn run ts-mocha -p ./tsconfig.json -t 1000000 tests/migrator.ts'
}

bankrun_timelock() {
    find programs tests sdk | entr -cs '(cd sdk && yarn build) && anchor build -p optimistic_timelock && RUST_LOG= yarn run ts-mocha -p ./tsconfig.json -t 1000000 tests/timelock.ts'
}

bankrun_vault_logs() {
    find programs tests sdk | entr -cs '(cd sdk && yarn build) && anchor build -p optimistic_timelock && RUST_LOG= yarn run ts-mocha -p ./tsconfig.json -t 1000000 tests/timelock.ts'
}

bankrun_logs() {
    (find programs && find tests) | entr -csr 'anchor build -p autocrat && yarn run ts-mocha -p ./tsconfig.json -t 1000000 tests/autocrat.ts'
}

case "$1" in
    test) test ;;
    test_no_build) test_no_build ;;
    build_verifiable) build_verifiable "$2" ;;
    deploy) deploy "$2" "$3" ;;
    upgrade) upgrade "$2" "$3" "$4" ;;
    upgrade_idl) upgrade_idl "$2" "$3" "$4" ;;
    bankrun) bankrun ;;
    test_amm) test_amm ;;
    test_amm_logs) test_amm_logs ;;
    bankrun_vault) bankrun_vault ;;
    bankrun_migrator) bankrun_migrator ;;
    bankrun_timelock) bankrun_timelock ;;
    bankrun_vault_logs) bankrun_vault_logs ;;
    bankrun_logs) bankrun_logs ;;
    *) echo "Unknown command: $1" ;;
esac