import { AddressLookupTableAccount, Connection, Keypair, PublicKey, SystemProgram, TransactionMessage, VersionedMessage, VersionedTransaction, LAMPORTS_PER_SOL } from '@solana/web3.js';
import fetch from 'cross-fetch';
import { Wallet } from '@coral-xyz/anchor';

const NOZOMI_URL = 'http://ams1.nozomi.temporal.xyz/';
const NOZOMI_UUID = process.env.NOZOMI_UUID;

// 0.001 SOL
const NOZOMI_TIP_LAMPORTS = 0.001 * LAMPORTS_PER_SOL;
const NOZOMI_TIP_ADDRESS = new PublicKey("TEMPaMeCRFAS9EKF53Jd6KpHxgL47uWLcpFArU1Fanq");

async function main() {
    const connection = new Connection(process.env.RPC_URL || 'https://api.mainnet-beta.solana.com', "confirmed");
    const nozomiConnection = new Connection(`${NOZOMI_URL}\?c=${NOZOMI_UUID}`);
    const privateKeyString = process.env.PRIVATE_KEY || '';
    const privateKeyArray = JSON.parse(privateKeyString);
    const privateKeyUint8 = new Uint8Array(privateKeyArray);
    const wallet = new Wallet(Keypair.fromSecretKey(privateKeyUint8));

    // Swapping SOL to USDC with input 0.1 SOL and 0.5% slippage
    const params = new URLSearchParams({
        inputMint: "So11111111111111111111111111111111111111112",
        outputMint: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
        amount: (0.1 * LAMPORTS_PER_SOL).toString(),
        // restrict intermediate tokens to only stable liquidity top set, minimizing high slippage risks with minimal pricing impact.
        restrictIntermediateTokens: 'true',
        slippageBps: (0.5 * 100).toString()
    })
    const quoteResponse = await (await fetch(`https://quote-api.jup.ag/v6/quote?${params}`)).json();

    // get serialized transactions for the swap
    const { swapTransaction } = await (
        await fetch('https://quote-api.jup.ag/v6/swap', {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json'
            },
            body: JSON.stringify({
                // quoteResponse from /quote api
                quoteResponse,
                // user public key to be used for the swap
                userPublicKey: wallet.publicKey.toString(),
                // auto wrap and unwrap SOL. default is true
                wrapAndUnwrapSol: true,
                // feeAccount is optional. Use if you want to charge a fee.  feeBps must have been passed in /quote API.
                // feeAccount: "fee_account_public_key"
            })
        })
    ).json();

    // deserialize the transaction
    const swapTransactionBuf = Buffer.from(swapTransaction, 'base64');
    const transaction = VersionedTransaction.deserialize(swapTransactionBuf);

    // tip transfer instruction
    let nozomiTipIx = SystemProgram.transfer({
        fromPubkey: wallet.publicKey,
        toPubkey: NOZOMI_TIP_ADDRESS,
        lamports: NOZOMI_TIP_LAMPORTS
    });

    // get the latest block hash
    let blockhash = await connection.getLatestBlockhash();

    let message = transaction.message;
    let addressLookupTableAccounts = await loadAddressLookupTablesFromMessage(message, connection);
    let txMessage = TransactionMessage.decompile(message, { addressLookupTableAccounts });

    txMessage.instructions.push(nozomiTipIx);

    let newMessage = txMessage.compileToV0Message(addressLookupTableAccounts);
    newMessage.recentBlockhash = blockhash.blockhash;

    let newTransaction = new VersionedTransaction(newMessage);
    
    // sign the transaction
    newTransaction.sign([wallet.payer]);

    // Execute the transaction
    const rawTransaction = newTransaction.serialize()
    let timestart = Date.now();
    const txid = await nozomiConnection.sendRawTransaction(rawTransaction, {
        skipPreflight: true,
        maxRetries: 2
    });

    console.log("Nozomi response: txid: %s", txid)

    let res = await connection.confirmTransaction({ signature: txid, blockhash: blockhash.blockhash, lastValidBlockHeight: blockhash.lastValidBlockHeight })
    console.log("Confirmed in: %s seconds", (Date.now() - timestart) / 1000)
}

async function loadAddressLookupTablesFromMessage(message: VersionedMessage, connection: Connection) {
    let addressLookupTableAccounts: AddressLookupTableAccount[] = [];
    for (let lookup of message.addressTableLookups) {
        let lutAccounts = await connection.getAddressLookupTable(lookup.accountKey);
        addressLookupTableAccounts.push(lutAccounts.value!);
    }

    return addressLookupTableAccounts;
}

main().catch(console.error);
