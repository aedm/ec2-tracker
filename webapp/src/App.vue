<script setup lang='ts'>
import {useFetchMarketplaceData} from '@/lib/fetchMarketplaceDataHook';
const {result, date} = useFetchMarketplaceData();

interface Column {
  title: string;
  key: string;
}

const columns: Column[] = [{
  title: 'Region',
  key: 'region',
}, {
  title: 'Count',
  key: 'count',
}, {
  title: 'Type',
  key: 'instance_type',
}, {
  title: 'Price',
  key: 'price',
}, {
  title: 'Hourly charge',
  key: 'recurring_charge',
}, {
  title: 'Months left',
  key: 'duration',
}, {
  title: 'OS',
  key: 'product_description',
}];

</script>

<template>
  <div class='container mx-auto mt-6'>
    Fetched at {{ date }}
    <table class='w-full text-sm text-left'>
      <tr>
        <th v-for="column in columns" :key="column.key">{{ column.title }}</th>
      </tr>
      <tr v-for='item in result' class="even:bg-gray-100 odd:bg-gray-200">
        <td v-for="column in columns" :key="column.key">{{ item[column.key] }}</td>
      </tr>
    </table>
  </div>
</template>
