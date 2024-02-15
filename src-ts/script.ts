import Chart, { ChartOptions } from 'chart.js/auto';
let errorSentence: HTMLSpanElement | null;
let nowPpm: HTMLSpanElement | null;
let chart: HTMLCanvasElement | null;
let chart2: any;

let address = import.meta.env.VITE_LOCAL_ADDRESS + ":" + import.meta.env.VITE_PORT;

window.addEventListener("DOMContentLoaded", () => {
  errorSentence = document.querySelector("#error");
  nowPpm = document.querySelector("#now-ppm");
  chart = document.querySelector("#chart");

  make_chart().then((result) => {
    change_display(result);
  }).catch((error) => {
    console.log(error);
  });

  setInterval(async () => {
    let result = await make_chart()
    change_display(result);
  }, 5000);
});

function change_display(result: boolean) {
  if (!result) {
    errorSentence!.style.display = "block";
  }
  else {
    errorSentence!.style.display = "none";
  };
}

async function make_chart(): Promise<boolean>{
  if(!nowPpm) return false;
  nowPpm.textContent = await invoke(address);
  if(!chart) return false;
  const ppm = Number(nowPpm.textContent);
  if(isNaN(ppm)) return false;
  let backgroundColor: string;
  if(ppm > 3500){
    backgroundColor = 'rgba(128, 0, 128, 1)';
  }
  else if(ppm > 2500){
    backgroundColor = 'rgba(255, 0, 0, 1)';
  }
  else if(ppm > 1500){
    backgroundColor = 'rgba(255, 165, 0, 1)';
  }
  else if(ppm > 1000){
    backgroundColor = 'rgba(0, 128, 0, 1)';
  }
  else{
    backgroundColor = 'rgba(0, 0, 255, 1)';
  }
  const percentage = Number(nowPpm.textContent) / 3500 * 100;
  if(isNaN(percentage)) return false;
  const donutOptions: ChartOptions = {
    borderColor: 'rgba(211, 211, 211, 1)',
    animation: false,
  }
  if(chart2 == null){
    chart2 = new Chart(chart, {
      type: 'doughnut',
      data: {
        datasets: [{
          data: [percentage, 100 - percentage],
          backgroundColor: [
            backgroundColor, //赤色
            'rgba(0, 0, 0, 0)',
          ],
        }]
      },
      options: donutOptions
    });
  }
  else{
    chart2.destroy();
    chart2 = new Chart(chart, {
      type: 'doughnut',
      data: {
        datasets: [{
          data: [percentage, 100 - percentage],
          backgroundColor: [
            backgroundColor, //赤色
            'rgba(0, 0, 0, 0)',
          ],
          animation: false,
        }]
      },
      options: donutOptions,
    });
  }

  nowPpm.textContent += " ppm";
  return true;
}

async function invoke(address: string): Promise<string> {
  try {
      const response = await fetch(`http://${address}/co2`);
      const text = await response.text();
      return text;
  } catch (e) {
      throw new Error(e as string);
  }
}
