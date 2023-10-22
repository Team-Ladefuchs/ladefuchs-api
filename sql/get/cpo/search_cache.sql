select  id,
        network,
        slug_name,
        updated,
        url,
        id as "cpo_id?"
from operator
where search @@ websearch_to_tsquery($1) or strict_word_similarity($1, slug_name) > 0.25
