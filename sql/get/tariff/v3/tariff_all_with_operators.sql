with query as (
    select distinct on (pub_tariff_id)
        pub_tariff_id as identifier,
        tariff.slug_name as name,
        provider_name,
        provider_customer_only as is_customer_only,
        monthly_fee,
        note,
        tariff.url as affiliate_link_url,
        tariff.updated as last_updated_date,
        ad_hoc as is_ad_hoc,
        case
            when image.soft_delete = false then $1 || 'image/' || image.checksum
        end as image_url,
        (

            o.pub_network = any($2)
            and tariff.monthly_fee = 0
            and tariff.provider_customer_only = false

        ) or tariff.standard as "is_standard!"
    from
        tariff
    left join image on tariff.image = image.id
    inner join public.charge_price as cp on tariff.id = cp.tariff_id
    inner join public.operator as o on cp.operator_id = o.id
    where
        tariff.hide = false
    order by
        pub_tariff_id
)

select *
from query
order by name, provider_name
