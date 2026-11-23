import fs from 'node:fs';
import path from 'node:path';
import {filterMap, isEmptyGuid, loadTranslations, PATHS, Table} from './common.lib.js';

/**
 * @typedef ActivityData
 * @type {object}
 * @property {number} _ManageIDSerial
 * @property {string} _MissionTextID
 * @property {string} _FlavorTextId
 * @property {boolean} _IsSideMission
 * @property {boolean} _IsExtraMission
 * @property {number} _ParentChapterId
 * @property {number} _ChildChapterId
 *
 * @typedef TextFile
 * @type {object}
 * @property {TextEntry[]} entries
 *
 * @typedef TextEntry
 * @type {object}
 * @property {string} name
 * @property {string} guid
 * @property {string[]} content
 *
 * @typedef MissionTextRecord
 * @type {object}
 * @property {string} fileId
 * @property {string} fileName
 * @property {string} filePath
 * @property {string} guid
 * @property {string} value
 */

const ACTIVITY_DATA_PATH = path.join(PATHS.data, 'missions/MissionActivityData.json');
const MISSION_TEXT_DIR = path.join(PATHS.text, 'missions/');

(() => {
    /**
     * @type {Array<ActivityData>}
     */
    const activityData = JSON.parse(fs.readFileSync(ACTIVITY_DATA_PATH));
    const textRecords = loadTranslations(MISSION_TEXT_DIR);

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
            width: 8,
        },
        {
            label: 'Title GUID',
            width: 36,
        },
        {
            label: 'Value',
            width: 24,
        },
    ]);

    const records = filterMap(activityData, v => {
        const record = textRecords.get(v._MissionTextID);

        if (record) {
            return [v, record];
        }
    });

    records.sort(([_a, a], [_b, b]) => a.value.localeCompare(b.value));

    let counter = 0;

    for (const [activity, record] of records) {
        table.row([
            (++counter).toString().padStart(3),
            record?.fileId ?? '–',
            activity._ManageIDSerial,
            activity._MissionTextID,
            record?.value ?? '–',
        ]);
    }
})();
