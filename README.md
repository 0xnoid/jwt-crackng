# jwt-crackng
Easy to use brute force cracker for JSON Web Tokens (JWT). Supports `HS256`, `HS384` & `HS512`.

<sub>Please note before using this software; This may be ineffective against Secrets with stronger protection.</sub>

### Inspiration
This project is inspired by/baeed on the pretty well established [jwt-cracker](https://github.com/lmammino/jwt-cracker) by [lmammino](https://github.com/lmammino) made using NodeJS, be sure to check it out.

#### What makes this one different?
Simply put: Rust. While we love NodeJS and the tool by lmammino, the performance leaves room for improvement, among other issues such as minimum-length. With Rust we get to fully utilize CPU, or GPU, to perform the task. There are also other improvements, so be sure to check the [features](#features)

# Features
- Wordlists
- Minimum and maximum length
- Built in Alphabet attack
- Load Management
- Output to file
- Input files (multiple JWTs)
- Install as a CLI tool (via alias and Bash handler)
- Autodetects signature, put full JWT or just the end
- Automatic updates (can be toggled)
- Container versions

# Installation
The easiest way to install this is to run the script (`Rust` & `Cargo` required):
```bash
```

For Docker:
```bash
```

# Usage
```bash
Usage: jwt-crackng [OPTIONS] --token <TOKEN>

Options:
  -t, --token <TOKEN>                 
  -o, --output <OUTPUT>               
  -n, --min-length <MIN_LENGTH>       [default: 1]
  -m, --max-length <MAX_LENGTH>       [default: 12]
  -u, --use-alphabet <ALPHABET>       [default: abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789]
  -l, --logfile <LOG_FILE>            
  -g, --all-guesses <ALL_TRIED_FILE>  
  -a, --algorithm <ALGORITHM>         [default: HS256] [possible values: HS256, HS384, HS512, HMACSHA256, HMACSHA384, HMACSHA512]
  -b, --base64                        
  -v, --verbose                       
      --gpu                           
      --gpu-limit <GPU_LIMIT>         
      --cpu <CPU>                     
      --ram <RAM>                     
      --cores <CORES>                 
      --limit <LIMIT>                 
      --dictionary <DICTIONARY>...    
  -h, --help                          Print help
```

Example (Default): 
```bash
jwt-crackng -t eyJhbGciOiJIUzI1NiJ9.eyJSb2xlIjoiQWRtaW4iLCJJc3N1ZXIiOiJJc3N1ZXIiLCJVc2VybmFtZSI6IkphdmFJblVzZSIsImV4cCI6MTczMzg3NDg2OSwiaWF0IjoxNzMzODc0ODY5fQ.CzXLrvPyf4IpZqUvQbU6xU507vevT8MKlqGhV5cUEu4
```


Example (Dictionary):
```bash
jwt-crackng -t eyJhbGciOiJIUzI1NiJ9.eyJSb2xlIjoiQWRtaW4iLCJJc3N1ZXIiOiJJc3N1ZXIiLCJVc2VybmFtZSI6IkphdmFJblVzZSIsImV4cCI6MTczMzg3NDg2OSwiaWF0IjoxNzMzODc0ODY5fQ.CzXLrvPyf4IpZqUvQbU6xU507vevT8MKlqGhV5cUEu4 -d pMerged.txt
```

# Other & Recommendations
This is a proof of concept tool that does not guarantee anything.


### Wordlist
We strongly recommend creating a wordlist for your situation. However, this is not always possible.\
The tool itself comes integrated with alphabet bruteforcing mode, however the success depends on the weakness of the key. The weaker the key is the easier it is to break.

The more you know about the backend and key or it's way of generating the higher your possibility of success will be.

### Time
If you have the resources and patience to run this program, feel free to. We do NOT recommend running this program against Tokens where you have zero knowledge, nor do we recommend running this program on hardware not suitable.

