import path from 'node:path';
import fs from 'node:fs';
import { loadTranslations, PATHS, Table, filterMap } from "./common.lib.js";

const MISSION_DATA_DIR = path.join(PATHS.data, 'missions/data');

/**
 * @typedef QuestLabels
 * @type {object}
 * @property {string} _TitleMsg
 * @property {number} _MissionId
 *
 * @typedef QuestData
 * @type {object}
 * @property {QuestLabels} _QuestMsg
 */
(() => {
    const textRecords = loadTranslations(path.join(PATHS.text, 'missions'));

    const table = new Table([
        {
            label: '#',
            width: 3,
        },
        {
            label: 'File ID',
            width: 6,
        },
        {
            label: 'Serial ID',
            width: 16,
        },
        {
            label: 'Title GUID',
            width: 36,
        },
        {
            label: 'Value',
            width: 32,
        },
    ]);

    const records = filterMap(fs.readdirSync(MISSION_DATA_DIR), fileName => {
        if (!fileName.endsWith('_QuestData.json')) {
            return undefined;
        }

        /** @type {QuestData} */
        const data = JSON.parse(fs.readFileSync(path.join(MISSION_DATA_DIR, fileName)));
        const textRecord = textRecords.get(data._QuestMsg._TitleMsg);

        if (!textRecord) {
            return undefined;
        }

        return [data, textRecord];
    });

    records.sort(([_a, a], [_b, b]) => a.value.localeCompare(b.value));

    let counter = 0;

    for (const [data, text] of records) {
        table.row([
            (++counter).toString().padStart(3),
            text.fileId,
            data._MissionId,
            data._QuestMsg._TitleMsg,
            text.value,
        ]);
    }
})();