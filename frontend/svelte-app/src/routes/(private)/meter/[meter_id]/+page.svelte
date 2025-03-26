<script lang="ts">
  import { SvelteMap } from 'svelte/reactivity';
  import { getContext, onMount } from 'svelte';

  import type { FluidMeterAlerts } from '@api/FluidMeter';
  import type { Message } from '@api/Message';
  import type { Series } from '@api/Common';

  import { FluidMeterStatus, activateFluidMeter, deactivateFluidMeter } from '@api/FluidMeter';
  import { MessageType } from '@api/Message';

  const globalMessages: SvelteMap<string, Message> = getContext('globalMessages');

  type Props = {
    data: {
      series: Series;
      alerts: FluidMeterAlerts;
    };
  };

  type chart_point = {
    date: string;
    litters: number;
  };

  let props: Props = $props();
  let alerts = $state(props.data.alerts);

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

  async function toggleStatus() {
    let r = null;
    let newStatus = FluidMeterStatus.Active;
    if (alerts.meter.status == FluidMeterStatus.Active) {
      r = await deactivateFluidMeter(alerts.meter.id);
      newStatus = FluidMeterStatus.Inactive;
    } else {
      r = await activateFluidMeter(alerts.meter.id);
    }

    if (r == 200) {
      alerts.meter.status = newStatus;
    } else {
      let message: Message = {
        type: MessageType.Error,
        text: 'Sorry. We failed to change the status.'
      };
      globalMessages.set('status-change-error', message);
    }
  }
</script>

<div class="container">
  <h1>{alerts.meter.name} ({alerts.meter.id})</h1>
  {#if alerts.alerts.length}
    <div class="alerts">
      <p><strong>Alerts</strong>:</p>
      {#each alerts.alerts as a}
        <span class="alert">{a.alert_type}</span>
      {/each}
    </div>
  {/if}
  <div class="st-container">
    <p><strong>Status</strong>: {alerts.meter.status}</p>
    <button class="button" onclick={() => toggleStatus()}>
      {alerts.meter.status == FluidMeterStatus.Active ? 'Deactivate' : 'Activate'}
    </button>
  </div>
  <p>Usage in the last 30 days</p>
  <div style="width: 800px;"><canvas id="usage"></canvas></div>
</div>

<style>
  .container {
    margin: 0 auto;
    margin-top: 1rem;
    width: 80%;
  }

  .st-container {
    display: flex;
  }

  .button {
    margin-left: 1rem;
  }

  .st-container p {
    margin: 0;
    line-height: 2rem;
  }

  .alerts {
    display: flex;
    margin-bottom: 1rem;
  }

  .alerts p {
    line-height: 2rem;
    margin: 0;
    padding: 0;
  }

  .alert {
    margin: 0 0.2rem;
    padding: 0.5rem;
    border: none;
    border-radius: 100px;
    background: #e3342f;
    color: #fff;
    font-weight: bold;
    font-size: 0.8em;
  }
</style>
