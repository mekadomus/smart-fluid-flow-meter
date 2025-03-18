<script lang="ts">
  import { onMount } from 'svelte';

  import type { Series } from '@api/Common';

  type Props = {
    data: {
      series: Series;
      meter_id: string;
    };
  };

  type chart_point = {
    date: string;
    litters: number;
  };

  let props: Props = $props();
  let meter_id = $state(props.data.meter_id);

  // We are getting data per hour for the last month ordered from newest to oldest.
  // Let's convert it to data per day from oldest to newest
  const data_arr: chart_point[] = [];
  let index = 0;
  let date = new Date(new Date().setHours(0, 0, 0, 0));
  for (let i = 0; i < 30; i++) {
    let data = {
      date: `${date.getDate()}/${date.getMonth() + 1}`,
      litters: 0
    };

    while (
      index < props.data.series.items.length &&
      new Date(props.data.series.items[index].period_start + 'Z') > date
    ) {
      data.litters += parseFloat(props.data.series.items[index].value);
      index++;
    }

    data_arr.push(data);
    date.setDate(date.getDate() - 1);
  }
  data_arr.reverse();

  onMount(async () => {
    const chartModule = await import('chart.js/auto');
    const chartjs = chartModule.Chart;

    const canvas = document.getElementById('usage') as HTMLCanvasElement;
    const ctx = canvas.getContext('2d');
    if (ctx) {
      new chartjs(ctx, {
        type: 'line',
        data: {
          labels: data_arr.map((v) => v.date),
          datasets: [
            {
              label: 'Litters',
              borderColor: 'rgb(12, 196, 247)',
              backgroundColor: 'rgb(12, 196, 247)',
              data: data_arr.map((v) => v.litters)
            }
          ]
        }
      });
    }
  });
</script>

<div class="container">
  <h1>{meter_id}</h1>
  <p>Usage in the last 30 days</p>
  <div style="width: 800px;"><canvas id="usage"></canvas></div>
</div>

<style>
  .container {
    margin: 0 auto;
    margin-top: 1rem;
    width: 80%;
  }
</style>
