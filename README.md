Hi! This is a little app intended to make picking vocabulary to learn easier.
Currently, only Russian is supported, but you should be able to change this by modifying only a couple of lines - I'll do it myself eventually.
You can insert a text or a text file on the Lemmatize tab. This will then be analyzed for its lemmas (the basic form you'd use to find a word in some dictionary).
Under Add, you can either add your own vocabulary cards, or you can add them semi-automatically by clicking on the "From queue" checkbox.
This will automatically insert the Russian word (from Lemmatize; the queue is ordered by frequency and first occurence in the corpus as well as the overall frequency in the language) and add a dictionary entry (from wiktionary).
That way, you only have to add the translation, which is much quicker than having to look up relevant words, inserting them in the vocabulary learning app of your choice, maybe adding an accent mark for pronunciation, switching keyboard layout back from Russian to English, etc.
If you already know a word or have some other reason not to study it, you can blacklist it. If you don't want to think about it just yet, you can ignore it - it won't be shown until you reopen the app again.

This app uses dumps from wiktionary - simply download the appropriate JSONL data from here: https://kaikki.org/.
If you want the app to also order by overall frequency, you will need to add a frequency text file like in this repository. The format is simple: the most frequent word is in the first line, the second most frequent word in the second line, etc.
You can add these files from the Main tab. Afterwards, you will need to create the tables for the database by clicking the appropraite buttons. This might take a minute or two (a checkmark will tell you when it's done).
The frequency table can only be created after the dictionary table.
Currently, there is no way to study the created cards in the app itself - rather, you can export them to Anki (simply click Export to create the file, then open Anki and import them to your deck).

This app is not for you if:
- you wish to study a language other than Russian (maybe come back in a month or two!)
- you already have a vocabulary deck you're satisfied with!
- you want to make cards that have more than one line per field or some fancy markdown (though I do plan to change that)

If you have any questions, please feel free to open an issue.
