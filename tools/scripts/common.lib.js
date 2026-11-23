import path from 'node:path';
import fs from 'node:fs';
import assert from 'node:assert';

export const PATHS = {
    projectRoot: path.join(import.meta.dirname, '../../'),
    data: path.join(import.meta.dirname, '../../output/user/'),
    text: path.join(import.meta.dirname, '../../output/msg/'),
};

/**
 * @typedef Header
 * @type {object}
 * @property {string} label
 * @property {number} width
 */
export class Table {
    /**
     *
     * @param {Header[]} headers
     * @param {number} spacing
     */
    constructor(headers, spacing = 2) {
        this.headers = headers.map(v => ({
            ...v,
            width: Math.max(v.width, v.label.length),
        }));

        this.spacing = spacing;

        this.printHeaders();
    }

    printHeaders() {
        /** @type {string[]} */
        const items = [];

        for (const header of this.headers) {
            items.push(header.label.padEnd(header.width));
        }

        console.log(items.join(' '.repeat(this.spacing)));
        console.log(items.map(v => '-'.repeat(v.length)).join(' '.repeat(this.spacing)));
    }

    /**
     *
     * @param {string[]} columns
     */
    row(columns) {
        assert(columns.length === this.headers.length, 'column count must be equal to header count');

        /** @type {string[]} */
        const items = [];

        for (let i = 0; i < columns.length; i++) {
            const padWidth = this.headers[i].width;
            items.push(columns[i].toString().padEnd(padWidth));
        }

        console.log(items.join(' '.repeat(this.spacing)));
    }
}

/**
 * @param {string} input
 * @returns {boolean}
 */
export function isEmptyGuid(input) {
    return input === '00000000-0000-0000-0000-000000000000';
}

/**
 *
 * @template T
 * @template V
 * @param {T[]} items
 * @param {(item: T) => V | undefined} filterFn
 * @returns {V[]}
 */
export function filterMap(items, filterFn) {
    /** @type {V[]} */
    const output = [];

    for (const item of items) {
        const result = filterFn(item);

        if (result !== undefined) {
            output.push(result);
        }
    }

    return output;
}

/**
 * @typedef TranslationFile
 * @type {object}
 * @property {TranslationEntry[]} entries
 *
 * @typedef TranslationEntry
 * @type {object}
 * @property {string} name
 * @property {string} guid
 * @property {string[]} content
 *
 * @typedef TranslationRecord
 * @type {object}
 * @property {string} fileId
 * @property {string} fileName
 * @property {string} filePath
 * @property {string} value
 *
 * @param {string} rootDir
 *
 * @returns {Map<string, TranslationRecord>}
 */
export function loadTranslations(rootDir) {
    /** @type {Map<string, TranslationRecord>} */
    const records = new Map();

    for (const fileName of fs.readdirSync(rootDir)) {
        if (!fileName.endsWith('.json')) {
            continue;
        }

        const filePath = path.join(rootDir, fileName);

        /** @type {TranslationFile} */
        const file = JSON.parse(fs.readFileSync(filePath));

        for (const entry of file.entries) {
            if (isEmptyGuid(entry.guid)) {
                continue;
            }

            records.set(entry.guid, {
                fileId: fileName.replace(/^Mission|\.json$/g, ''),
                fileName,
                filePath,
                value: entry.content[1],
            });
        }
    }

    return records;
}