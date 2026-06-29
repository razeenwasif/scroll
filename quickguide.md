# Scroll TUI Command Cheatsheet

Scroll is your read-it-later dashboard and knowledge vault. Run it by typing `scroll` in your terminal.

---

## 🧭 Library View Navigation
* **`Left` / `Right`** (or **`h` / `l`** / **`Enter`**): Toggle keyboard focus between the **Folders Sidebar** (left) and the **Articles List** (right).
* **`Up` / `Down`** (or **`j` / `k`**): Scroll through the categories or article list.
* **`Tab`**: Cycle status filters at the top: `All` ➔ `Unread` ➔ `Reading` ➔ `★ Favorites` ➔ `Archived`.
* **`Enter`** (on article list): Open the selected article in the reading viewport.
* **`Esc`** or **`q`** (in reading viewport): Close the article and return to the library.
* **`/`**: Focus the **Search query box** to filter articles immediately.

## ⌨️ TUI Hotkeys (Normal Mode)
* **`a`**: Open the command line pre-populated with `:add ` to scrape a URL.
* **`c`**: Open the command line pre-populated with `:category ` to organize the selected article.
* **`d`**: Archive the selected article.
* **`f`**: Toggle Favorite star (`★`).
* **`s`**: Cycle sorting: `Date Created` ➔ `Title` ➔ `Reading Progress` ➔ `Date Updated`.
* **`1` / `2` / `3` / `4`**: Switch modes globally:
  1. `1` ➔ Library View
  2. `2` ➔ Reader View (if article is open)
  3. `3` ➔ Spaced Repetition Flashcards Review
  4. `4` ➔ Highlights & Annotations Vault

## 💬 Command Line Mode (`:`)
Press `:` in Library normal mode to activate the command prompt:
* **`:q` / `:quit`**: Exit Scroll.
* **`:help`**: Print quick help status banner.
* **`:add <url>`**: Clip the URL in the background, clean the HTML, extract content, and add it to your library.
* **`:category <name>`**: Assign the selected article to a folder category (e.g. `Research Papers`). Type `clear`, `none`, or leave empty to unassign.
* **`:export`**: Export all saved library articles into your workspace under `/home/amaterasu/Scroll/exports/` as raw `.md` files.

## 🧠 Flashcard Review Session Mode
* **`Space`**: Flip the card to reveal the answer.
* **`1` - `5`**: Rate your retention quality using the SM-2 algorithm:
  * `1` (Hardest) ➔ `5` (Easiest). Schedules the next review date dynamically.
* **`s`**: Skip card.
* **`q`**: Terminate the review session.
