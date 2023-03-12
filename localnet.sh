#!/bin/sh

anchor build && solana-test-validator --bpf-program Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS target/deploy/account_history_program.so --reset
