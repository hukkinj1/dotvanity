use bip39;
use clap;
use data_encoding::HEXLOWER;
use sp_core::crypto::AccountId32;
use sp_core::crypto::Ss58AddressFormat;
use sp_core::crypto::Ss58Codec;
use sp_core::Pair;

fn is_valid_ss58_char(c: char) -> bool {
    let ss58_chars = [
        '1', '2', '3', '4', '5', '6', '7', '8', '9', 'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'J',
        'K', 'L', 'M', 'N', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z', 'a', 'b', 'c',
        'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v',
        'w', 'x', 'y', 'z',
    ];
    ss58_chars.contains(&c)
}

struct Matcher {
    addr_type: u8,
    startswith: String,
    endswith: String,
}

impl Matcher {
    fn match_(&self, candidate: &String) -> bool {
        if !candidate.starts_with(&self.startswith) {
            return false;
        }
        if !candidate.ends_with(&self.endswith) {
            return false;
        }
        true
    }
    fn validate(&self) {
        if !self.startswith.chars().all(is_valid_ss58_char)
            || !self.endswith.chars().all(is_valid_ss58_char)
        {
            eprintln!("Error: A provided matcher contains SS58 incompatible characters");
            std::process::exit(1);
        }

        // Validate first char of --startswith string for some known cases
        if !self.startswith.is_empty() {
            let first_char = self.startswith.chars().next().unwrap();
            if self.addr_type == 0 && first_char != '1' {
                eprintln!(
                    "Error: Polkadot mainnet address must start with '1'. Adjust --startswith"
                );
                std::process::exit(1);
            }
            let kusama_addr_first_chars = ['C', 'D', 'F', 'G', 'H', 'J'];
            if self.addr_type == 2 && !kusama_addr_first_chars.contains(&first_char) {
                eprintln!("Error: Kusama address must start with one of ['C', 'D', 'F', 'G', 'H', 'J']. Adjust --startswith");
                std::process::exit(1);
            }
            if self.addr_type == 42 && first_char != '5' {
                eprintln!(
                    "Error: Generic Substrate address must start with '5'. Adjust --startswith"
                );
                std::process::exit(1);
            }
        }
    }
}

struct Wallet {
    mnemonic_phrase: String,
    private_key: String,
    public_key: String,
    address: String,
}

fn generate_wallet(addr_format: u8) -> Wallet {
    let mnemonic = bip39::Mnemonic::new(bip39::MnemonicType::Words12, bip39::Language::English);
    let phrase = mnemonic.phrase();
    let (pair, secret) = sp_core::sr25519::Pair::from_phrase(phrase, None).unwrap();
    let address_obj = AccountId32::from(pair.public());
    let address_str = address_obj.to_ss58check_with_version(Ss58AddressFormat::Custom(addr_format));
    Wallet {
        mnemonic_phrase: phrase.to_string(),
        private_key: HEXLOWER.encode(&secret),
        public_key: pair.public().to_string(),
        address: address_str,
    }
}

fn main() {
    let matches = clap::App::new("dotvanity")
        .version("0.1.1")  // DO NOT EDIT THIS LINE MANUALLY
        .author("Taneli Hukkinen <hukkinj1@users.noreply.github.com>")
        .about("Polkadot/Substrate vanity address generator")
        .arg(
            clap::Arg::with_name("startswith")
                .short("s")
                .long("startswith")
                .value_name("SUBSTRING")
                .help("A string that the address must start with")
                .default_value(""),
        )
        .arg(
            clap::Arg::with_name("endswith")
                .short("e")
                .long("endswith")
                .value_name("SUBSTRING")
                .help("A string that the address must end with")
                .default_value(""),
        )
        .arg(
            clap::Arg::with_name("address type")
                .short("t")
                .long("type")
                .value_name("INT")
                .help("Address type. Should be an integer value in range 0 to 127.\n\
                          Notable types:\n\
                          \t0 - Polkadot mainnet\n\
                          \t2 - Kusama network\n\
                          \t42 - Generic Substrate\n\
                          Defaults to Polkadot mainnet. For more types, see \
                          https://github.com/paritytech/substrate/wiki/External-Address-Format-(SS58)#address-type")
                .default_value("0"),  // Polkadot mainnet type
        )
        .get_matches();

    let addr_type_str = matches.value_of("address type").unwrap();
    let addr_type: u8 = match addr_type_str.parse() {
        Ok(addr_type) => addr_type,
        Err(_error) => {
            eprintln!("Error: Address type is not an 8-bit unsigned integer");
            std::process::exit(1);
        }
    };
    if addr_type > 127 {
        eprintln!("Error: Address type must be in range [0, 127]");
        std::process::exit(1);
    }

    let matcher = Matcher {
        addr_type: addr_type,
        startswith: String::from(matches.value_of("startswith").unwrap()),
        endswith: String::from(matches.value_of("endswith").unwrap()),
    };
    matcher.validate();

    let mut wallet: Wallet;
    loop {
        wallet = generate_wallet(addr_type);
        if matcher.match_(&wallet.address) {
            break;
        }
    }

    println!("Mnemonic phrase: {}", wallet.mnemonic_phrase);
    println!("Private key: {}", wallet.private_key);
    println!("Public key: {}", wallet.public_key);
    println!("Address: {}", wallet.address);
}
