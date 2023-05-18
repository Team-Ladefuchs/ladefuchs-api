select cpo_cache.id,
        cpo_cache.network,
        cpo_cache.slug_name,
        cpo_cache.updated,
        url,
        cpo.id as "cpo_id?"
from cpo_cache left join cpo on cpo_cache.network = cpo.network
where search @@ websearch_to_tsquery($1) or strict_word_similarity($1, cpo_cache.slug_name) > 0.25
