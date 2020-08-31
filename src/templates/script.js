const searchIndex = fetch('search-index.json').then(response => response.json())

/**
 * A function to compute the Levenshtein distance between two strings
 * Licensed under the Creative Commons Attribution-ShareAlike 3.0 Unported
 * Full License can be found at http://creativecommons.org/licenses/by-sa/3.0/legalcode
 * This code is an unmodified version of the code written by Marco de Wit
 * and was found at http://stackoverflow.com/a/18514751/745719
 */
var levenshtein_row2 = [];
function levenshtein(s1, s2) {
    if (s1 === s2) {
        return 0;
    }
    var s1_len = s1.length, s2_len = s2.length;
    if (s1_len && s2_len) {
        var i1 = 0, i2 = 0, a, b, c, c2, row = levenshtein_row2;
        while (i1 < s1_len) {
            row[i1] = ++i1;
        }
        while (i2 < s2_len) {
            c2 = s2.charCodeAt(i2);
            a = i2;
            ++i2;
            b = i2;
            for (i1 = 0; i1 < s1_len; ++i1) {
                c = a + (s1.charCodeAt(i1) !== c2 ? 1 : 0);
                a = row[i1];
                b = b < a ? (b < c ? b + 1 : c) : (a < c ? a + 1 : c);
                row[i1] = b;
            }
        }
        return b;
    }
    return s1_len + s2_len;
}

async function search(input) {
    const index = await searchIndex
    const scoredItems = []

    for (const item of index) {
        const scoredItem = scoreIndexItem(input, item)
        scoredItems.push(scoredItem)
    }

    scoredItems.sort(sortScoredItem)
    return scoredItems.map(item => item[1]).slice(0, 20)
}

const INDEX = 0
const NAME = 1
const TYPE = 2
const PARENT_NAME = 3
const PARENT_TYPE = 4

function scoreIndexItem(input, item) {
    const index = item[INDEX]
    let min = Infinity

    for (const v of index) {
        const val = levenshtein(input, v)
        if (val < min) {
            min = val
        }
    }

    return [min, item]
}

function sortItem(a, b) {
    if (a[NAME] < b[NAME]) {
        return -1
    }
    else {
        return 1
    }
}

function sortScoredItem(a, b) {
    if (a[0] !== b[0]) {
        return a[0] - b[0]
    }

    return sortItem(a[1], b[1])
}

const searchElement = document.querySelector('[name=search]')
const bodyWrapper = document.querySelector('#body_wrapper')
const searchContent = document.querySelector('#search_content')
const main = document.querySelector('#main')
searchElement.addEventListener('input', async e => {
    const input = searchElement.value.trim()
    if (input == '') {
        bodyWrapper.setAttribute('data-state', 'main')
    }
    else {
        bodyWrapper.setAttribute('data-state', 'search')
        const items = await search(input)

        while (searchContent.lastChild) {
            searchContent.removeChild(searchContent.lastChild)
        }

        for (item of items) {
            searchContent.appendChild(renderItem(item))
        }
    }
}, false)

function renderItem(item) {
    const h3 = document.createElement('h3')
    const code = document.createElement('code')
    h3.appendChild(code)

    if (item[PARENT_NAME]) {
        // This is a child
        const parentLink = document.createElement('a')
        parentLink.href = `${item[PARENT_TYPE]}.${item[PARENT_NAME]}.html`
        parentLink.innerText = item[PARENT_NAME]
        parentLink.classList.add(item[PARENT_TYPE])
        const childLink = document.createElement('a')
        childLink.href = `${item[PARENT_TYPE]}.${item[PARENT_NAME]}.html#${item[TYPE]}.${item[NAME]}`
        childLink.innerText = item[NAME]
        childLink.classList.add(item[TYPE])
        const textContent = document.createTextNode('.')
        code.appendChild(parentLink)
        code.appendChild(textContent)
        code.appendChild(childLink)
    }
    else {
        // This is not a child
        const link = document.createElement('a')
        link.href = `${item[TYPE]}.${item[NAME]}.html`
        link.innerText = item[NAME]
        link.classList.add(item[TYPE])
        code.appendChild(link)
    }

    return h3
}
