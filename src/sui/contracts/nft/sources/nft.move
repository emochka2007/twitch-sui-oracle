module nft::nft;

use std::string;
use sui::url::{Self, Url};

public struct EmoNFT has key, store {
    id: UID,
    name: string::String,
    description: string::String,
    url: Url
}

public fun name(nft: &EmoNFT): &string::String {
    &nft.name
}

/// Get the NFT's `description`
public fun description(nft: &EmoNFT): &string::String {
    &nft.description
}

/// Get the NFT's `url`
public fun url(nft: &EmoNFT): &Url {
    &nft.url
}


#[allow(lint(self_transfer))]
public fun mint_to_sender(
    name: vector<u8>,
    description: vector<u8>,
    url: vector<u8>,
    ctx: &mut TxContext
) {
    let sender = ctx.sender();
    let nft = EmoNFT {
        id: object::new(ctx),
        name: string::utf8(name),
        description: string::utf8(description),
        url: url::new_unsafe_from_bytes(url)
    };

    transfer::public_transfer(nft, sender);
}

public fun transfer(nft: EmoNFT, recipient: address, _ctx: &mut TxContext) {
    transfer::public_transfer(nft, recipient);
}

public fun update_description(
    nft: &mut EmoNFT,
    new_description: vector<u8>,
    _ctx: &mut TxContext
) {
    nft.description = string::utf8(new_description);
}

public fun burn(nft: EmoNFT, _ctx: &mut TxContext) {
    let EmoNFT { id, name: _, description: _, url: _ } = nft;
    id.delete();
}