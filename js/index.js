require("dotenv").config();
const {
  PublicKey,
  Keypair,
  Connection,
  Transaction,
  SystemProgram,
  LAMPORTS_PER_SOL,
  TransactionInstruction,
  sendAndConfirmTransaction,
} = require("@solana/web3.js");
const {
  Token,
  TOKEN_PROGRAM_ID,
} = require("@solana/spl-token");
const BN = require("bn.js");

const ECHO_SPACE = 32;

const callEchoIx = async (connection, programId, feePayer, args) => {
  const echo = Keypair.generate();
  let echoKey = echo.publicKey;
  let tx = new Transaction();
  let signers = [feePayer];

  if(args.length < 3) {
    console.log("Generatiing new echo address");
    let createIx = SystemProgram.createAccount({
      fromPubkey: feePayer.publicKey,
      newAccountPubkey: echoKey,
      lamports: await connection.getMinimumBalanceForRentExemption(ECHO_SPACE),
      space: ECHO_SPACE,
      programId: programId,
    });
    signers.push(echo);
    tx.add(createIx);
  } else {
    echoKey = new PublicKey(args[2]);
  }

  const idx = Buffer.from(new Uint8Array([0]));
  const msg = Buffer.from(args[1]);
  const len = Buffer.from(new Uint8Array(new BN(args[1].length).toArray("le", 4)));

  let echoIx = new TransactionInstruction({
    keys: [
      {
        pubkey: echoKey,
        isSigner: false,
        isWritable: true,
      }
    ],
    programId: programId,
    data: Buffer.concat([idx, len, msg]),
  });

  tx.add(echoIx);

  console.log("Sending Tx");
  let txID = await sendAndConfirmTransaction(connection, tx, signers, {
    skipPrelight: true,
    preflightCommitment: "confirmed",
    confirmation: "confirmed",
  });
  console.log(`https://explorer.solana.com/tx/${txID}?cluster=devnet`);

  data = (await connection.getAccountInfo(echoKey)).data;
  // buf = new BN(data, "le");
  console.log("Echo Key:", echoKey.toBase58());
  console.log("Msg: ", data.toString());
}


const callAuthEchoIx = async (connection, programId, feePayer, args) => {
  const buf_seed = new Uint8Array(new BN(Math.pow(3, 7)).toArray("le", 8));
  let enc = new TextEncoder();
  const [echoKey, bump] = await PublicKey.findProgramAddress(
    [
      enc.encode("authority"),
      feePayer.publicKey.toBytes(),
      buf_seed,
    ],
    programId,
  );

  let tx = new Transaction();
  let signers = [feePayer];

  let initAuthIx = new TransactionInstruction({
    keys: [
      {
        pubkey: echoKey,
        isSigner: false,
        isWritable: true,
      },
      {
        pubkey: feePayer.publicKey,
        isSigner: true,
        isWritable: false,
      },
      {
        pubkey: SystemProgram.programId,
        isSigner: false,
        isWritable: false,
      }
    ],
    programId: programId,
    data: Buffer.concat([
      Buffer.from(new Uint8Array([1])),
      // Buffer.from(new Uint8Array(new BN(9).toArray("le", 4))),
      Buffer.from(buf_seed),
      Buffer.from(new Uint8Array(new BN(ECHO_SPACE).toArray("le", 8)))
    ])
  });

  tx.add(initAuthIx);

  const msg = Buffer.from(args[1]);
  const len = Buffer.from(new Uint8Array(new BN(args[1].length).toArray("le", 4)));

  let authEchoIx = new TransactionInstruction({
    keys: [
      {
        pubkey: echoKey,
        isSigner: false,
        isWritable: true,
      },
      {
        pubkey: feePayer.publicKey,
        isSigner: true,
        isWritable: false,
      }
    ],
    programId: programId,
    data: Buffer.concat([Buffer.from(new Uint8Array([2])), len, msg]),
  });

  tx.add(authEchoIx);

  console.log("Sending Tx");
  let txID = await sendAndConfirmTransaction(connection, tx, signers, {
    skipPrelight: true,
    preflightCommitment: "confirmed",
    confirmation: "confirmed",
  });
  console.log(`https://explorer.solana.com/tx/${txID}?cluster=devnet`);

  data = (await connection.getAccountInfo(echoKey)).data;
  // buf = new BN(data, "le");
  console.log("Echo Key:", echoKey.toBase58());
  console.log("Msg: ", data.toString());
}

const callVendingEchoIx = async (connection, programId, feePayer, args) => {
  const mint = await Token.createMint(connection, feePayer, feePayer.publicKey, null, 0, TOKEN_PROGRAM_ID);
  const tok_acnt = await mint.createAssociatedTokenAccount(feePayer.publicKey);
  await mint.mintTo(tok_acnt, feePayer, [], 10000);
  const price = new Uint8Array(new BN(Math.pow(10, 3)).toArray("le", 8));;


  let enc = new TextEncoder();
  const [echoKey, bump] = await PublicKey.findProgramAddress(
    [
      enc.encode("vending machine"),
      mint.publicKey.toBytes(),
      price,
    ],
    programId,
  );

  let tx = new Transaction();
  let signers = [feePayer];

  let initVendIx = new TransactionInstruction({
    keys: [
      {
        pubkey: echoKey,
        isSigner: false,
        isWritable: true,
      },
      {
        pubkey: mint.publicKey,
        isSigner: false,
        isWritable: false,
      },
      {
        pubkey: feePayer.publicKey,
        isSigner: true,
        isWritable: false,
      },
      {
        pubkey: SystemProgram.programId,
        isSigner: false,
        isWritable: false,
      }
    ],
    programId: programId,
    data: Buffer.concat([
      Buffer.from(new Uint8Array([3])),
      // Buffer.from(new Uint8Array(new BN(9).toArray("le", 4))),
      Buffer.from(price),
      Buffer.from(new Uint8Array(new BN(ECHO_SPACE).toArray("le", 8)))
    ])
  });

  tx.add(initVendIx);

  const msg = Buffer.from(args[1]);
  const len = Buffer.from(new Uint8Array(new BN(args[1].length).toArray("le", 4)));

  let authEchoIx = new TransactionInstruction({
    keys: [
      {
        pubkey: echoKey,
        isSigner: false,
        isWritable: true,
      },
      {
        pubkey: feePayer.publicKey,
        isSigner: true,
        isWritable: false,
      },
      {
        pubkey: tok_acnt,
        isSigner: false,
        isWritable: true,
      },
      {
        pubkey: mint.publicKey,
        isSigner: false,
        isWritable: true,
      },
      {
        pubkey: TOKEN_PROGRAM_ID,
        isSigner: false,
        isWritable: false,
      }
    ],
    programId: programId,
    data: Buffer.concat([Buffer.from(new Uint8Array([4])), len, msg]),
  });

  tx.add(authEchoIx);

  console.log("Sending Tx");
  let txID = await sendAndConfirmTransaction(connection, tx, signers, {
    skipPrelight: true,
    preflightCommitment: "confirmed",
    confirmation: "confirmed",
  });
  console.log(`https://explorer.solana.com/tx/${txID}?cluster=devnet`);

  data = (await connection.getAccountInfo(echoKey)).data;
  let bal = await connection.getTokenAccountBalance(tok_acnt);
  // buf = new BN(data, "le");
  console.log("Echo Key:", echoKey.toBase58());
  console.log("Msg: ", data.toString());
  console.log(bal.value.uiAmount);
}

const main = async () => {
  // console.log(Keypair.generate())
  let args = process.argv.slice(2);
  //args[0]: Message
  const connection = new Connection(process.env.SOLANA_PROVIDER_URL);
  // const feePayer = Keypair.fromSecretKey(new Uint8Array(
  //   [
  //     174, 204,   7, 231,  22, 191, 165, 200,  48, 153, 200,
  //      14,  58,  80,  22, 139,   6, 205, 187, 197,  16, 115,
  //      15, 179, 236, 196, 156,  60, 210, 152, 136, 223, 143,
  //     162, 162,  45, 138,  34, 251, 227, 119, 202, 176, 141,
  //     145, 205,   0, 146, 243, 239, 250, 156,  36,  54,  35,
  //      56, 145, 153,  46,  14, 121,   9,  86, 140
  //   ]
  // ));
  const feePayer = Keypair.generate();
  const programId = new PublicKey(process.env.PROGRAM_ID);
// connection.getTokenAccountBalance()
  const aTx = await connection.requestAirdrop(feePayer.publicKey, 1 * LAMPORTS_PER_SOL);
  await connection.confirmTransaction(aTx);

  if(args[0] == "0"){
    console.log("0")
    await callEchoIx(connection, programId, feePayer, args);
  } else if(args[0] == "1"){
    console.log("1 2");
    await callAuthEchoIx(connection, programId, feePayer, args)
  } else if(args[0] == "3"){
    console.log("3 4");
    await callVendingEchoIx(connection, programId, feePayer, args);
  }
}

main()
  .then(() => {
    console.log("Success");
  })
  .catch((e) => {
    console.error(e);
  });
