#!/usr/bin/env node

const fetch = require('node-fetch')
const fs = require('fs').promises
const docql = require('./pkg')

class Runtime {
    async date() {
        return new Date().toISOString().split('T').shift()
    }

    async getArgs() {
        return process.argv.slice(2)
    }

    async query(url, graphql, headers) {
        const all_headers = Object.assign({}, headers, { 'content-type': 'application/json' });
        const response = await fetch(url, {
            method: 'POST',
            body: JSON.stringify(graphql),
            headers: all_headers,
        })

        return await response.json()
    }

    async prepareOutputDirectory(output) {
        await fs.mkdir(output, { recursive: true })
    }

    async writeFile(output, file, contents) {
        await fs.writeFile(`${output}/${file}`, contents)
    }
}


async function main() {
    const runtime = new Runtime()
    const response = await docql.main(runtime)
}

main().catch(err => {
    console.error(err.message)
    process.exit(err.exitCode || 1)
})
