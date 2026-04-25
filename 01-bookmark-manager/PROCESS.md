# What I built & why: 

Built a one-page, local bookmark manager, for guild review; a small project to sample development with an ai agent

# Process
1) presented design doc and initial prompt

prompt: i want to build the bookmark manager from my design doc. start with a single index.html file with a form to add bookmarks (url and title) and a list underneath that shows them. Save them to localstorage so they stay when i refresh. plain html, css, and javascript. newest first in list. add validation to make sure all necessary parts are filled as designed.

verified: it worked right away, didn't have to tweak it

2) "The core bookmark list is working. Now I want to add two fields to the add form: an optional note (a text area for a brief description) and optional tags (comma-separated text that gets stored as a list). Display the note under each bookmark's title, and display tags as small labeled badges."

verified: they display as designed

3) "Now add the ability to edit and delete bookmarks. Each bookmark should have a small edit icon and a delete icon. Clicking edit should let me change the title, URL, note, and tags inline. Clicking delete should remove the bookmark after a confirmation. Make sure changes persist in local storage."

verified: buttons work as tested

4) "Now add tag filtering. Above the bookmark list, display all unique tags as clickable buttons. When I click a tag, the list should filter to only show bookmarks with that tag. There should be an 'All' button that removes the filter. The currently active filter should be visually highlighted."

verified. tag tabs work; different bookmarks display for what they're tagged

5) "Add a search bar next to the tag filters. It should filter the bookmark list in real time as I type, matching against bookmark titles and notes. Search and tag filters should work together. If I have a tag filter active and then search, it should search within the filtered results."

verified. tested extra by creating a bookmark while actively searching. new bookmark appeared to show search works entirely as designed

6) "The functionality is complete. Now let's polish the interface. I want: better spacing and typography, a subtle color scheme (dark background with light text, dark mode), smooth transitions when filtering and searching, the add form should collapse to a button when not in use to keep the interface clean, and bookmarks should show the domain name extracted from the URL as a subtle label."

initially, produced a page where the form appeared regardless of whether it's supposed to be collapsed or expanded

followup: "the form isn't collapsing to a button, it's always open"

verified working as desired

# What I learned
First time i used a design doc, I learned it makes development a LOT easier, and faster. Adding more to the design was easier in waves aswell. Like building a building, start with the foundation and add each floor as layers while you build up. The only bug in the process was at the end with collapsing the form when unneeded. Which was very easy to rectify.

# Known Issues
- The manager is only dark theme, when it started out light theme. A light/dark theme toggle would be better.
- A way to import/export bookmarks would also be nice, especially since localstorage could be wiped out; export/import would be great to safeguard against that.
- Tags aren't deduplicated. 