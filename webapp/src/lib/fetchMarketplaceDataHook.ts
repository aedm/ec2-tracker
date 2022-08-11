import axios, {AxiosResponse} from 'axios';
import {asType} from '@/lib/asType';
import {ref, Ref} from 'vue';

interface OfferResponse {
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
    result: Ref<OfferResponse[]>;
    date: Ref<string>
}

const DATA_FILES_BASE_URL = 'https://ec2-scraper.s3.amazonaws.com';

function processOfferList() {

}



export function useFetchMarketplaceData(): FetchHook {
    const result = ref(asType<OfferResponse[]>([]));
    const date = ref('');

    async function fetchData() {
        const urlResult = await axios.get(`${DATA_FILES_BASE_URL}/latest.txt`);
        const fileName = urlResult.data;
        console.log("Fetch URL", fileName);
        const contentResult = await axios.get(`${DATA_FILES_BASE_URL}/${fileName}`);
        console.log("Fetch content", contentResult.data.length);
        const table = contentResult.data as OfferResponse[];
        for (let row of table) {
            row.duration = Math.round(row.duration / (60 * 60 * 24 * 30) * 10) / 10;
        }
        table.sort((a, b) => a.instance_type.localeCompare(b.instance_type));
        date.value = fileName.substring(3, fileName.length - 8);
        result.value = table;
    }

    const _ = fetchData();

    return {
        result,
        date,
    }
}
