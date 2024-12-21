use std::str::FromStr;

use borsh::{BorshDeserialize, BorshSerialize};
use solana_client::rpc_client::RpcClient;
use solana_program::pubkey::Pubkey;
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    signature::{read_keypair_file, Keypair},
    signer::Signer,
    transaction::Transaction,
};
#[derive(BorshDeserialize, BorshSerialize, Debug)]
pub struct UserProfile {
    pub data_len: u16,
    pub follows: Vec<Pubkey>,
}

#[derive(BorshDeserialize, BorshSerialize, Debug)]
pub struct UserPost {
    pub post_count: u64,
    pub posts: Vec<Post>,
}

#[derive(BorshDeserialize, BorshSerialize, Debug)]
pub struct Post {
    pub content: String,
    pub timestamp: u64,
}
#[derive(BorshDeserialize, BorshSerialize, Debug)]
pub enum SocialInstruction {
    InitializeUser { seed_type: String },
    FollowUser { user_to_follow: Pubkey },
    UnfollowUser { user_to_unfollow: Pubkey },
    QueryFollower,
    PostContent { content: String },
    QueryPosts,
}

impl UserProfile {
    pub fn new() -> Self {
        Self {
            data_len: 0,
            follows: Vec::new(),
        }
    }

    pub fn follow(&mut self, user: Pubkey) {
        self.follows.push(user);
        self.data_len = self.follows.len() as u16;
    }

    pub fn un_follow(&mut self, user: Pubkey) {
        self.follows.retain(|&x| x != user);
        self.data_len = self.follows.len() as u16;
    }
}

impl Post {
    pub fn new(content: String, timestamp: u64) -> Self {
        Self { content, timestamp }
    }
}
impl UserPost {
    pub fn new() -> Self {
        Self {
            post_count: 0,
            posts: Vec::new(),
        }
    }
    pub fn post(&mut self, post: Post) {
        self.posts.push(post);
        self.post_count = self.posts.len() as u64;
    }
    pub fn query_posts(&self) -> &Vec<Post> {
        self.posts.as_ref()
    }
}
const USER_PROFILE_SEED: &str = "profile";
const USER_POST_SEED: &str = "post";
pub struct SocialClient {
    rpc_client: RpcClient,
    program_id: Pubkey,
}

impl SocialClient {
    pub fn new(rpc_url: &str, program_id: Pubkey) -> Self {
        let rpc_client = RpcClient::new(rpc_url.to_string());
        Self {
            rpc_client,
            program_id,
        }
    }
    pub fn initialize_user(
        &self,
        user_keypair: &Keypair,
        seed_type: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let pda = get_pda(
            &self.program_id,
            &[user_keypair.pubkey().as_ref(), seed_type.as_ref()],
        );
        let initialize_user_instruction = Instruction::new_with_borsh(
            self.program_id,
            &SocialInstruction::InitializeUser {
                seed_type: seed_type.to_string(),
            },
            vec![
                AccountMeta::new(user_keypair.pubkey(), true),
                AccountMeta::new(pda, false),
                AccountMeta::new_readonly(solana_sdk::system_program::id(), false),
            ],
        );
        self.send_instruction(user_keypair, vec![initialize_user_instruction])?;
        Ok(())
    }
    pub fn follow_user(
        &self,
        user_keypair: &Keypair,
        follow_user: Pubkey,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let pda = get_pda(
            &self.program_id,
            &[user_keypair.pubkey().as_ref(), USER_PROFILE_SEED.as_ref()],
        );
        let follow_user_instruction: Instruction = Instruction::new_with_borsh(
            self.program_id,
            &SocialInstruction::FollowUser {
                user_to_follow: follow_user,
            },
            vec![AccountMeta::new(pda, false)],
        );
        self.send_instruction(user_keypair, vec![follow_user_instruction])?;
        Ok(())
    }

    pub fn qurey_followers(
        &self,
        user_keypair: &Keypair,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let pda = get_pda(
            &self.program_id,
            &[user_keypair.pubkey().as_ref(), USER_PROFILE_SEED.as_ref()],
        );
        let query_follower_instruction: Instruction = Instruction::new_with_borsh(
            self.program_id,
            &SocialInstruction::QueryFollower,
            vec![AccountMeta::new(pda, false)],
        );
        self.send_instruction(user_keypair, vec![query_follower_instruction])?;
        Ok(())
    }
    pub fn delete_followers(
        &self,
        user_keypair: &Keypair,
        user_to_unfollow: Pubkey,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let pda = get_pda(
            &self.program_id,
            &[user_keypair.pubkey().as_ref(), USER_PROFILE_SEED.as_ref()],
        );
        let delete_follower_instruction: Instruction = Instruction::new_with_borsh(
            self.program_id,
            &SocialInstruction::UnfollowUser {
                user_to_unfollow: (user_to_unfollow),
            },
            vec![AccountMeta::new(pda, false)],
        );
        self.send_instruction(user_keypair, vec![delete_follower_instruction])?;
        Ok(())
    }

    pub fn qurey_posts(&self, user_keypair: &Keypair) -> Result<(), Box<dyn std::error::Error>> {
        let pda = get_pda(
            &self.program_id,
            &[user_keypair.pubkey().as_ref(), USER_POST_SEED.as_ref()],
        );
        let query_follower_instruction: Instruction = Instruction::new_with_borsh(
            self.program_id,
            &SocialInstruction::QueryPosts,
            vec![AccountMeta::new(pda, false)],
        );
        self.send_instruction(user_keypair, vec![query_follower_instruction])?;
        Ok(())
    }

    pub fn send_posts(
        &self,
        user_keypair: &Keypair,
        content: String,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let pda = get_pda(
            &self.program_id,
            &[user_keypair.pubkey().as_ref(), USER_POST_SEED.as_ref()],
        );
        let send_post_instruction: Instruction = Instruction::new_with_borsh(
            self.program_id,
            &SocialInstruction::PostContent { content },
            vec![AccountMeta::new(pda, false)],
        );
        self.send_instruction(user_keypair, vec![send_post_instruction])?;
        Ok(())
    }
    pub fn send_instruction(
        &self,
        payer: &Keypair,
        instructions: Vec<Instruction>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let last_blockhash = self.rpc_client.get_latest_blockhash()?;
        let transaction = Transaction::new_signed_with_payer(
            &instructions,
            Some(&payer.pubkey()),
            &[payer],
            last_blockhash,
        );
        let signature = self.rpc_client.send_and_confirm_transaction(&transaction)?;
        println!("signature is {:?}", signature);
        Ok(())
    }
}
fn get_pda(program_id: &Pubkey, seed: &[&[u8]]) -> Pubkey {
    let (pda, _bump) = Pubkey::find_program_address(seed, program_id);
    println!("pda is {}", pda);
    pda
}
fn main() {
    //calculate_data_size();
    let program_id = "53W1m3utd9wBMAThwa2RR7v4DkXiapbjUG9BUcDkv9WM";
    users_profile_test(program_id).unwrap();
    users_post_test(program_id).unwrap();
}

fn users_post_test(program_id_str: &str) -> Result<(), Box<dyn std::error::Error>> {
    let user_keypair = read_keypair_file("/Users/cdbb/.config/solana/id.json").expect("fail");
    let program_id = Pubkey::from_str(program_id_str).unwrap();
    let client = SocialClient::new("http://127.0.0.1:8899", program_id);

    //1. initializing user psot
    // client.initialize_user(&user_keypair, USER_POST_SEED)?;
    // client.qurey_posts(&user_keypair)?;
    //2. send post
    //let content = "12345678901234567890";
    let content = "09876543210987654321";
    client.send_posts(&user_keypair, content.to_string())?;
    client.qurey_posts(&user_keypair)?;

    Ok(())
}

fn users_profile_test(program_id_str: &str) -> Result<(), Box<dyn std::error::Error>> {
    let user_keypair = read_keypair_file("/Users/cdbb/.config/solana/id.json").expect("fail");
    let program_id = Pubkey::from_str(program_id_str).unwrap();
    let client = SocialClient::new("http://127.0.0.1:8899", program_id);

    //1. initializing user profile
    client.initialize_user(&user_keypair, USER_PROFILE_SEED)?;
    //2. follow user
    let follow_user = Pubkey::from_str("BXrAqseotsUN2iQtGYnZqvbsu3N5RJYPmrRQ6BZDJXM5")?;
    client.follow_user(&user_keypair, follow_user)?;

    let follow_user = Pubkey::from_str("ABCAqseotsUN2iQtGYnZqvbsu3N5RJYPmrRQ6BZDJXM5")?;
    client.follow_user(&user_keypair, follow_user)?;
    //3. query follows
    client.qurey_followers(&user_keypair)?;
    //4. delete follower
    let unfollow_user = Pubkey::from_str("BXrAqseotsUN2iQtGYnZqvbsu3N5RJYPmrRQ6BZDJXM5")?;
    client.delete_followers(&user_keypair, unfollow_user)?;
    Ok(())
}

fn _calculate_data_size() {
    let user_profile = UserPost::new();
    println!(
        "user_profile len is {:?}",
        borsh::to_vec(&user_profile).unwrap().len()
    );
    let content = "1234567890123456789012345678901234567890".to_string();
    let post = Post::new(content.clone(), 1 as u64);

    println!(
        "content len is {:?}",
        borsh::to_vec(&content).unwrap().len()
    );
    println!("ts len is {:?}", borsh::to_vec(&(1 as u64)).unwrap().len());
    println!("post len is {:?}", borsh::to_vec(&post).unwrap().len());
}
