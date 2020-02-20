use std::fs;
use std::process::exit as Exit;
use std::env;
use std::time;
extern crate dirs;
extern crate yaml_rust;


struct Context {
    name: String,
    cluster: String,
    namespace: String,
    user: String,
}

#[derive(Default)]
struct Cluster {
    name: String,
    server: String,
    certificate_authority: String,
}

#[derive(Default)]
struct AuthProvider {
    name: String,
    client_id: String,
    client_secret: String,
    id_token: String,
    idp_issuer_url: String,
    refresh_token: String,
}

#[derive(Default)]
struct User {
    name: String,
    client_cert: String,
    client_key: String,
    auth_provider: AuthProvider,
}

fn main() {

    let home_dir = match dirs::home_dir() {
        Some(path) => path.into_os_string(),
        None => {
            println!("Didn't see a home directory for the current user, aborting..");
            Exit(1);
        }
    };

    let args: Vec<String> = env::args().collect();
    // println!("{:#?}", args);

    if args.len() < 2 {
        // println!("Missing required argument: <name of kubeconfig context to use>, defaulting to listing available contexts..");
        let kubeconfig = fs::read_to_string(format!("{}{}", home_dir.to_str().unwrap(), "/.kube/config")).expect("Could not read file @ ~/.kube/config");
        // println!("{}", &kubeconfig);
        let data = yaml_rust::YamlLoader::load_from_str(&kubeconfig).expect("Unable to extract data from config stream");
        list_contexts(&data[0]);
        Exit(2)
    }
    

    let out_yaml;
    let kubeconfig = fs::read_to_string(format!("{}{}", home_dir.to_str().unwrap(), "/.kube/config")).expect("Could not read file @ ~/.kube/config");
    // println!("{}", &kubeconfig);
    let data = yaml_rust::YamlLoader::load_from_str(&kubeconfig).expect("Unable to extract data from config stream");
    let my_ctx = find_context(args[1].as_str(), &data[0]).expect("Couldn't find a context with the search term");
    let my_cluster = find_cluster(my_ctx.cluster.as_str(), &data[0]).expect("Couldn't find a cluster based on context reference");
    let my_user = find_user(my_ctx.user.as_str(), &data[0]).expect("Couldn't find a user based on the context reference");
    //println!("Found context '{}', using user '{}' in namespace '{}' on cluster '{}'", my_ctx.name, my_ctx.user, my_ctx.namespace, my_ctx.cluster);
    //println!("Found master url '{}' for referenced cluster '{}'", my_cluster.server, my_ctx.cluster);
    if my_user.client_cert != "" {
        //println!("Found user '{}' with cert location '{}' and key location '{}'", my_user.name, my_user.client_cert, my_user.client_key);
        out_yaml = format!("
    apiVersion: v1
    kind: Config
    clusters:
    - cluster: 
        certificate-authority: {}
        server: {}
      name: {}
    contexts:
    - context:
        cluster: {}
        namespace: {}
        user: {}
      name: {}
    users: 
    - user: 
        client-certificate: {}
        client-key: {}
      name: {}
    current-context: {}
    ",my_cluster.certificate_authority, my_cluster.server,my_cluster.name, my_ctx.cluster, my_ctx.namespace, my_ctx.user, my_ctx.name, my_user.client_cert, my_user.client_key, my_user.name, my_ctx.name);
    } else {
        //println!("Found user '{}' with auth-provider '{}'", my_user.name, my_user.auth_provider.name);
        out_yaml = format!("
    apiVersion: v1
    kind: Config
    clusters:
    - cluster: 
        certificate-authority: {}
        server: {}
      name: {}
    contexts:
    - context:
        cluster: {}
        namespace: {}
        user: {}
      name: {}
    users: 
    - user: 
        auth-provider: 
          config: 
            client-id: {}
            client-secret: {}
            id-token: {}
            idp-issuer-url: {}
            refresh-token: {}
          name: {}
      name: {}
    current-context: {}
    ",my_cluster.certificate_authority, my_cluster.server,my_cluster.name, my_ctx.cluster, my_ctx.namespace, my_ctx.user, my_ctx.name, my_user.auth_provider.client_id, my_user.auth_provider.client_secret, my_user.auth_provider.id_token, my_user.auth_provider.idp_issuer_url, my_user.auth_provider.refresh_token, my_user.auth_provider.name, my_user.name, my_ctx.name);
    }

    let start_time = time::SystemTime::now();
    let since_epoch = start_time.duration_since(time::UNIX_EPOCH).expect("Wut?");
    let rando_file_part = since_epoch.as_millis();
    let out_doc = yaml_rust::YamlLoader::load_from_str(&out_yaml).expect("Unable to load built YAML..");
    let mut out_file_content = String::new();
    let mut emitter = yaml_rust::YamlEmitter::new(&mut out_file_content);
    emitter.dump(&out_doc[0]).unwrap();
    fs::write(format!("{}.{}", "/tmp/kubeconfig", rando_file_part), &out_file_content).expect("File could not be written!");
    println!("export KUBECONFIG={}", format!("{}.{}", "/tmp/kubeconfig", rando_file_part));
    
}

fn list_contexts(kubeconfig: &yaml_rust::Yaml) {
    let count = kubeconfig["contexts"].as_vec().unwrap().len();
    
    for i in 0..count {
        println!("{}", kubeconfig["contexts"][i]["name"].as_str().unwrap());
    }
}

fn find_context(s: &str, kubeconfig: &yaml_rust::Yaml) -> Option<Context> {
    let count = kubeconfig["contexts"].as_vec().unwrap().len();
    let mut context = Context {
        name: String::from("unknown"),
        cluster:String::from("unknown"),
        namespace: String::from("unknown"),
        user: String::from("unknown"),
    };
    // println!("Searching {} contexts for target {}", count, s);
    
    for i in 0..count {
        if kubeconfig["contexts"][i]["name"].as_str().unwrap() == s {
            // println!("{:#?}", kubeconfig["contexts"][i]["context"].as_hash().unwrap());
            context = Context{
                name: String::from(s),
                cluster: String::from(kubeconfig["contexts"][i]["context"]["cluster"].as_str().unwrap()),
                namespace: String::from(kubeconfig["contexts"][i]["context"]["namespace"].as_str().unwrap()),
                user: String::from(kubeconfig["contexts"][i]["context"]["user"].as_str().unwrap()),
            };
          
        } 
    }
    if context.name == "unknown" {
        Option::from(None)
    } else {
        Option::from(context)
    }
    
}

fn find_cluster(s: &str, kubeconfig: &yaml_rust::Yaml) -> Option<Cluster> {
    let count = kubeconfig["clusters"].as_vec().unwrap().len();
    let mut cluster = Cluster::default();
    // println!("Searching {} clusters for target {}", count, s);
    
    for i in 0..count {
        if kubeconfig["clusters"][i]["name"].as_str().unwrap() == s {
            // println!("{:#?}", kubeconfig["contexts"][i]["context"].as_hash().unwrap());
            cluster.name = String::from(s);
            cluster.server= String::from(kubeconfig["clusters"][i]["cluster"]["server"].as_str().unwrap());
            cluster.certificate_authority = String::from(kubeconfig["clusters"][i]["cluster"]["certificate-authority"].as_str().unwrap());
            
        } 
    }
    if cluster.name == "" {
        Option::from(None)
    } else {
        Option::from(cluster)
    }

}

fn find_user(s: &str, kubeconfig: &yaml_rust::Yaml) -> Option<User> {
    let count = kubeconfig["users"].as_vec().unwrap().len();
    let mut user = User::default();
    // println!("Searching {} users for target {}", count, s);
    
    for i in 0..count {
        if kubeconfig["users"][i]["name"].as_str().unwrap() == s {
            //println!("{:#?}", kubeconfig["users"][i]["user"].as_hash().unwrap());
            if !kubeconfig["users"][i]["user"]["auth-provider"].is_null() {
                user.name = String::from(s);
                user.auth_provider.name = String::from(kubeconfig["users"][i]["user"]["auth-provider"]["name"].as_str().unwrap());
                user.auth_provider.client_id = String::from(kubeconfig["users"][i]["user"]["auth-provider"]["config"]["client-id"].as_str().unwrap());
                user.auth_provider.client_secret = String::from(kubeconfig["users"][i]["user"]["auth-provider"]["config"]["client-secret"].as_str().unwrap());
                user.auth_provider.id_token = String::from(kubeconfig["users"][i]["user"]["auth-provider"]["config"]["id-token"].as_str().unwrap());
                user.auth_provider.idp_issuer_url = String::from(kubeconfig["users"][i]["user"]["auth-provider"]["config"]["idp-issuer-url"].as_str().unwrap());
                user.auth_provider.refresh_token = String::from(kubeconfig["users"][i]["user"]["auth-provider"]["config"]["refresh-token"].as_str().unwrap());
            } else {
                user.name = String::from(s);
                user.client_cert = String::from(kubeconfig["users"][i]["user"]["client-certificate"].as_str().unwrap());
                user.client_key = String::from(kubeconfig["users"][i]["user"]["client-key"].as_str().unwrap());
            }
        } 
    }
    if user.name == "" {
        Option::from(None)
    } else {
        Option::from(user)
    }
}