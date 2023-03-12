import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { OracleHistoryProgram } from "../target/types/oracle_history_program";

describe("oracle-history-program", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.OracleHistoryProgram as Program<OracleHistoryProgram>;

  it("Is initialized!", async () => {
    // Add your test here.
    const tx = await program.methods.initialize().rpc();
    console.log("Your transaction signature", tx);
  });
});
