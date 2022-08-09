<script setup lang='ts'>
import {useFetchMarketplaceData} from '@/lib/fetchMarketplaceDataHook';
const {result, date} = useFetchMarketplaceData();
</script>

<template>
  <div class='container mx-auto mt-6'>
    Fetched at {{ date }}
    <table class='w-full text-sm text-left'>
      <tr>
        <th>region</th>
        <th>type</th>
        <th>price</th>
        <th>hourly</th>
        <th>months</th>
        <th>count</th>
        <th>os</th>
      </tr>
      <tr v-for='item in result' class="even:bg-gray-100 odd:bg-gray-200">
        <td>{{ item.region }}</td>
        <td>{{ item.instance_type }}</td>
        <td>{{ item.price }}</td>
        <td>{{ item.recurring_charge }}</td>
        <td>{{ item.duration / (60 * 60 * 24 * 30) }}</td>
        <td>{{ item.count }}</td>
        <td>{{ item.product_description }}</td>
      </tr>
    </table>
  </div>
</template>
