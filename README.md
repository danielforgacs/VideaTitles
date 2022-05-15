## Videa movie title scraper

```
$ videatitles <number-of-pages>
```

- Prints a list of title, url pairs in the terminal.
- it's looking for the `.videablacklist.txt` file in the `home` directory.
    if it doesn't exist it will be created. Movie titles are checked if
    they contain every any in the blacklist and if yes, it's not listed.
