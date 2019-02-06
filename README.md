# talk

A program that utilizes ngrams to generate new sentences. Input is given as a list of sentences which are then broken up into ngrams of length 4. The ngrams are stored compactly to take up as little space as possible, so it is plausible to run this in the background given your dataset isn't too large.

# Examples
## Getting a single generated sentence
To generate one sentence, pass in your dataset (or load a pre-compiled set), and it will output one sentence. By default, it will try to load files from `./data/sentences`

To pass a dataset, use the `-d` flag. You can then pass a directory which has separate files for different sets; for example, one file per book, user, etc.

`talk -d ./data`

## Compiling a dataset
There is an option to compile a dataset which makes it faster to load. This is ideal if you will be drawing from the same dataset for multiple runs. To compile a directory, run the following.

`talk -d ./data -c data.dat`

You can then load the data next run by using:

`talk -l data.dat`

## Running an IPC server (Linux only)
There is an option to run this program as an IPC server via the `-s` flag. What this does is create a socket, `ipc:///tmp/talk.ipc`, which can be queried for sentences. This will leave the program running in the background like a service.

To query a sentence, pass the string "gen" to the socket. It will then return a generated sentence.

`talk -s`

## Stats
You can print the statistics for a loaded dataset by passing the `--stats` flag.

`talk -l data.dat --stats`

## Minimum required sources
If you have multiple sources, it might be desirable to make sure talk pulls from a certain amount of them to avoid pulling a sentence straight from a single source. To do this, pass the `-u` flag with an integer argument. This will set the minimum amount of unique sources it must pull from to create a valid generated sentence. The default is 1.

`talk -l data.dat -u 4`
