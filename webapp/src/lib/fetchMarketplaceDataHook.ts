import axios, {AxiosResponse} from 'axios';
import {asType} from '@/lib/asType';
import {ref, Ref} from 'vue';

export interface Offer {
    id: string,
    region: string,
    count: number,
    instance_type: string,
    price: number,
    recurring_charge: number,
    duration: number,
    fixed_price: number,
    availability_zone: string,
    product_description: string,
    usage_price: number,
    instance_tenancy: string,
    offering_class: string,
    offering_type: string,
    scope: string
}

interface FetchHook {
    result: Ref<Offer[]>;
    date: Ref<string>
}

const BASE_URL = 'https://ec2-scraper.s3.amazonaws.com';

export function useFetchMarketplaceData(): FetchHook {
    const result = ref(asType<Offer[]>([]));
    const date = ref('');

    async function search() {
        const urlResult = await axios.get(`${BASE_URL}/latest.txt`);
        const fileName = urlResult.data;
        console.log("Fetch URL", fileName);
        const contentResult = await axios.get(`${BASE_URL}/${fileName}`);
        console.log("Fetch content", contentResult.data.length);
        const table = contentResult.data as Offer[];
        for (let row of table) {
            row.duration = Math.round(row.duration / (60 * 60 * 24 * 30) * 10) / 10;
        }
        table.sort((a, b) => a.instance_type.localeCompare(b.instance_type));
        date.value = fileName.substring(3, fileName.length - 8);
        result.value = table;
    }

    const _ = search();

    return {
        result,
        date,
    }
}
