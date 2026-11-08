import json
from pathlib import Path
import sqlite3
import tempfile
import unittest
from scripts.judgment_data import export, pack, stats


class ExportTest(unittest.TestCase):
    def test_latest_recognition_skip_and_readonly(self):
        with tempfile.TemporaryDirectory() as temp:
            path = Path(temp)/'server.db'
            with sqlite3.connect(path) as db:
                db.executescript((Path(__file__).resolve().parents[1]/'server/sql/judgments.sql').read_text())
                db.execute('INSERT INTO judgment_case VALUES(?,?,?,?,?,?)',
                    ('case', 'group', 'play', 'left', json.dumps({'view': {'kind': 'play'}, 'context': {'complete': True}, 'source_split': 0}), 0))
                for user in ['alice', 'bob', 'carol']:
                    db.execute('INSERT INTO judgment_assignment(user_id,case_id,group_id) VALUES(?,?,?)', (user,'case','group'))
                for user,rev,status,recognized in [('alice',1,'answered',False),('alice',2,'answered',False),('bob',1,'answered',True),('carol',1,'skipped',False)]:
                    db.execute('INSERT INTO judgment_response(case_id,user_id,revision,status,payload) VALUES(?,?,?,?,?)',
                        ('case',user,rev,status,json.dumps({'status':status,'recognized':recognized,'action':['AH'],'label_semantics':'expert_recommendation'})))
            before = path.read_bytes()
            out = Path(temp)/'labels.jsonl'
            self.assertEqual(export(path, out)['exported'],1)
            row=json.loads(out.read_text())
            self.assertEqual(row['revision'],2)
            self.assertNotEqual(row['reviewer'],'alice')
            self.assertTrue(row['question']['context']['complete'])
            self.assertEqual(export(path,out,True,True)['exported'],2)
            self.assertEqual(json.loads(out.read_text().splitlines()[0])['reviewer'],'alice')
            self.assertEqual(stats(path)['revisions'],4)
            self.assertEqual(path.read_bytes(),before)
            with self.assertRaises(ValueError): export(path,path)

    def test_pack_preserves_analysis_context_but_not_reviews(self):
        with tempfile.TemporaryDirectory() as temp:
            path=Path(temp)/'local.db'; out=Path(temp)/'questions.jsonl'
            case={'id':'id','schema':'turbo-hearts-judgment-v1','view':{},'context':{},'selection':{}}
            with sqlite3.connect(path) as db:
                db.execute('CREATE TABLE cases(id,queue_order,payload)')
                db.execute('INSERT INTO cases VALUES(?,?,?)',('id',0,json.dumps(case)))
            self.assertEqual(pack(path,out)['questions'],1)
            self.assertEqual(json.loads(out.read_text()),case)


if __name__=='__main__': unittest.main()
