import sys
import os

try:
    import invenio
except ImportError:
    pass
else:
    from invenio.dbquery import run_sql as q
    from invenio.search_engine import perform_request_search as inv_search
    from invenio.search_engine import perform_request_search
    from invenio.search_engine import search_unit, search_pattern
    from invenio.bibrank_citation_searcher import (get_citation_dict,
                                                   get_cited_by,
                                                   get_refers_to)
    from invenio.search_engine import get_record
    from invenio.search_engine_utils import get_fieldvalues
