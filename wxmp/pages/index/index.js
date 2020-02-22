const app = getApp()

const DEVICE_NAME = 'ThumbPiano';

const MAX_BYTES = 18;

const MUSICS = new Map([
  ['两只老虎', '4,90_c,d,e,c|c,d,e,c|e,f,g,-|e,f,g,-|g6,a2,g6,f2,e,c|g6,a2,g6,f2,e,c|c,5,c,-|c,5,c,-'],
  ['新年好', '3,92_0,0,c4,c4|c,5,e4,e4|e,c,c4,e4|g,g,f4,e4|d,-,d4,e4|f,f,e4,d4|e,c,c4,e4|d,5,74,d4|c,-_0,0,0|1,0,0|1,0,0|1,5,0|2,5,0|2,4,0|1,0,0|2,0,0|1,5'],
]);

var connecting = false;
var find_time = 0;

var totalLen = 0;
var sendLen = 0;

/**
 * Shuffles array in place. ES6 version
 * @param {Array} a items An array containing the items.
 */
function shuffle(a) {
  for (let i = a.length - 1; i > 0; i--) {
    const j = Math.floor(Math.random() * (i + 1));
    [a[i], a[j]] = [a[j], a[i]];
  }
  return a;
}

function inArray(arr, key, val) {
  for (let i = 0; i < arr.length; i++) {
    if (arr[i][key] === val) {
      return i;
    }
  }
  return -1;
}

// 字符串转byte
function stringToBytes(str) {
  var array = new Uint8Array(str.length);
  for (var i = 0, l = str.length; i < l; i++) {
    array[i] = str.charCodeAt(i);
  }
  return array.buffer;
}

// ArrayBuffer转16进度字符串示例
function ab2hex(buffer) {
  var hexArr = Array.prototype.map.call(
    new Uint8Array(buffer),
    function (bit) {
      return ('00' + bit.toString(16)).slice(-2)
    }
  )
  return hexArr.join('');
}

// 16进制数转ASCLL码
function hexCharCodeToStr(hexCharCodeStr) {
  var trimedStr = hexCharCodeStr.trim();
  var rawStr = trimedStr.substr(0, 2).toLowerCase() === "0x" ? trimedStr.substr(2) : trimedStr;
  var len = rawStr.length;
  var curCharCode;
  var resultStr = [];
  for (var i = 0; i < len; i = i + 2) {
    curCharCode = parseInt(rawStr.substr(i, 2), 16);
    resultStr.push(String.fromCharCode(curCharCode));
  }
  return resultStr.join("");
}

Page({
  data: {
    status: '蓝牙已关闭', //状态指示
    musics: Array.from(MUSICS.keys()),  //所有音乐名列表
    musicIndex: 0,                      //当前选择的音乐
    musicString: MUSICS.get(MUSICS.keys().next().value),  //当前音乐的串
    musicSections: [],//歌曲小节索引数组
    startSection: 0,  //起始小节
    endSection: 0, //结束小节
    loop: false,
    device: null,
    recvMessage: '',
    connected: false,
  },
  onLoad(){
    this.onMusicChange({ detail: {
      value: 0
    } });
  },
  //开始搜索名称为ThumbPiano的设备
  openBluetoothAdapter() {
    if (this.data.connected){
      this.setData({ status: '设备已连接!' });
      return;
    }
    if (this._discoveryStarted) {
      this.setData({ status: '正在搜索设备' });
      return;
    }
    wx.openBluetoothAdapter({
      success: (res) => {
        console.log('蓝牙打开成功，开始搜索设备...', res)
        this.setData({ status: '蓝牙已打开' });
        this.startBluetoothDevicesDiscovery()
      },
      fail: (res) => {
        this.setData({ status: '蓝牙打开失败' });
        console.log('蓝牙打开失败:', res);
        if (res.errCode === 10001) {
          wx.onBluetoothAdapterStateChange(function (res) {
            console.log('onBluetoothAdapterStateChange', res)
            if (res.available) {
              this.startBluetoothDevicesDiscovery()
            }
          })
        }
      }
    })
  },
  getBluetoothAdapterState() {
    wx.getBluetoothAdapterState({
      success: (res) => {
        console.log('getBluetoothAdapterState', res)
        if (res.discovering) {
          this.onBluetoothDeviceFound()
        } else if (res.available) {
          this.startBluetoothDevicesDiscovery()
        }
      }
    })
  },
  startBluetoothDevicesDiscovery() {
    if (this._discoveryStarted) {
      this.setData({ status: '正在搜索设备' });
      return
    }
    this._discoveryStarted = true
    wx.startBluetoothDevicesDiscovery({
      allowDuplicatesKey: true,
      success: (res) => {
        this.setData({ status: '正在搜索设备' });
        console.log('设备搜索已开启', res)
        //注册监听函数
        this.onBluetoothDeviceFound()
      },
    })
  },
  stopBluetoothDevicesDiscovery() {
    wx.stopBluetoothDevicesDiscovery()
  },
  onBluetoothDeviceFound() {
    var that = this;
    //注册监听函数
    wx.onBluetoothDeviceFound((res) => {
      res.devices.forEach(device => {
        console.log("设备名称:"+device.name+" "+device.localName);
        find_time += 1;
        if (device.name && device.name == DEVICE_NAME){
          that.stopBluetoothDevicesDiscovery();
          //找到蓝牙开始连接
          that.setData({ status: "连接" + DEVICE_NAME});
          that.createBLEConnection(device);
          return;
        }
        if(find_time > 20){
          that.stopBluetoothDevicesDiscovery();
          that.closeBluetoothAdapter();
          that.setData({ status: DEVICE_NAME+'设备未找到' });
        }
      })
    })
  },
  //连接设备
  createBLEConnection(ds) {
    if (connecting){
      return;
    }
    connecting = true;
    var that = this;
    const deviceId = ds.deviceId
    const name = ds.name
    this.setData({
      deviceId: deviceId
    });

    wx.createBLEConnection({
      deviceId,
      success: (res) => {
        this.setData({
          status: DEVICE_NAME+"已连接",
          connected: true,
          device: ds,
          name,
          deviceId,
        })

        wx.getBLEDeviceServices({
          deviceId,
          success: (res) => {
            var service = res.services[1];
            let serviceId = service.uuid;
            that.setData({
              serviceId: serviceId
            });
            console.log('serviceId=', serviceId);
            wx.getBLEDeviceCharacteristics({
              deviceId,
              serviceId: serviceId,
              success: (res) => {
                for (var i = 0; i < res.characteristics.length; i++) {//2个值
                  var model = res.characteristics[i]
                  if (model.properties.notify == true) {
                    that.setData({
                      notifyId: model.uuid//监听的值
                    })
                    that.startNotice(model.uuid)//7.0
                  }
                  if (model.properties.write == true) {
                    that.setData({
                      status: DEVICE_NAME + "已连接, 可写",
                      writeId: model.uuid//用来写入的值
                    });
                  }
                }
              },
              fail(res) {
                console.error('getBLEDeviceCharacteristics fail:', res)
              }
            });
          }
        })
      },
      fail: function (res) {
        that.closeBluetoothAdapter();
        that.setData({
          status: '设备连接失败'
        });
      }
    });
  },
  startNotice(uuid) {
    var that = this;
    //开始监听
    wx.notifyBLECharacteristicValueChange({
      state: true, // 启用 notify 功能
      deviceId: that.data.deviceId,
      serviceId: that.data.serviceId,
      characteristicId: uuid,
      success: function (res) {
        //成功监听，注册接收函数
        wx.onBLECharacteristicValueChange(function (res) {
          // 此时可以拿到蓝牙设备返回来的数据是一个ArrayBuffer类型数据，所以需要通过一个方法转换成字符串
          var hex = ab2hex(res.value);
          var text = hexCharCodeToStr(hex);
          console.log('接收到数据:', text);
          that.setData({ recvMessage: DEVICE_NAME+'发来消息:'+text});
        });
      }
    });
  },
  checkWrite(){
    if (!this.data.writeId) {
      wx.showToast({
        icon: "none",
        title: '设备未连接!',
      });
      this.setData({
        status: '设备未连接!'
      });
      return false;
    }
    this.setData({
      status: '正在传输'
    });
    return true;
  },
  //发送MAX_BYTES字节
  sendTextByte(text, cb){
    if (!this.checkWrite()) {
      return;
    }
    var that = this;
    var buffer = stringToBytes(text);
    wx.writeBLECharacteristicValue({
      deviceId: that.data.deviceId,
      serviceId: that.data.serviceId,
      characteristicId: that.data.writeId,
      value: buffer,
      success: function (res) {
        console.log(text, "写入成功");
      },
      fail: function () {
        console.log('写入失败');
      },
      complete: function (res) {
        cb();
      }
    });
  },
  //将字符串写入到蓝牙设备当中
  sendMessage(text){
    if (!this.checkWrite()) {
      return;
    }
    
    var that = this;
    if (text.length <= MAX_BYTES) {
      this.sendTextByte(text, function () {
        that.setData({
          status: '传输完成'
        });
        wx.showToast({
          title: '传输完成',
        });
        sendLen = 0;
        totalLen = 0;
        console.log('发送完毕!');
      });
    } else {
      //一次传输MAX_BYTES字节
      var chars = text.split('');
      let subchars = chars.splice(0, MAX_BYTES);
      
      wx.showToast({
        title: '已发送' + parseInt((sendLen / totalLen) * 100)+"%",
        icon: 'loading',
        mask: true
      });
      //这里用+分割，否则stm32连续接收字符串会丢失数据
      this.sendTextByte(subchars.join('')+'+', function(){
        sendLen += subchars.length;
        that.sendMessage(chars.join(''));
      });
    }
  },
  //选择音乐
  onMusicChange(data){
    let index = data.detail.value;
    let musicNames = Array.from(MUSICS.keys());
    var musicString = MUSICS.get(musicNames[index]);
    if(musicString == ''){
      this.randomMusic();
      return;
    }
    //剔除空格
    musicString = musicString.replace(/ /g, '');
    //计算所有小节
    let strArrays = musicString.split('_');
    let prefix = strArrays[0];
    let theme = strArrays[1];
    let accompany = strArrays[2];
    
    let sectionNum = theme.split('|').length;
    var musicSections = [];
    for(var i=1; i<=sectionNum; i++){
      musicSections.push(i);
    }

    //设置bpm
    let maxbpm = parseInt(prefix.split(',')[1]);
    let bpm = 0;
    //节拍数最多减半
    let musicBpm = [];
    for(var i=maxbpm; i>=2; i-=1){
      musicBpm.push(i);
    }

    this.setData({
      bpm: 0,
      musicBpm: musicBpm,
      musicIndex: index,
      musicString: musicString,
      musicSections: musicSections,
      startSection: 0,
      endSection: sectionNum-1,
    });
  },
  onBpmChange(data){
    let index = data.detail.value;
    this.setData({
      bpm: index
    });
  },
  //修改起始节
  onStartSectionChange(data){
    let index = data.detail.value;
    this.setData({
      startSection: index
    });
  },
  //修改结束节
  onEndSectionChange(data){
    let index = data.detail.value;
    this.setData({
      endSection: index
    });
  },
  //关闭设备连接
  closeBLEConnection() {
    wx.closeBLEConnection({
      deviceId: this.data.deviceId
    });
    this.setData({
      connected: false,
      canWrite: false,
    });
    connecting = false;
  },
  //开始练习
  startPlay(){
    let index = this.data.musicIndex;
    let musicNames = Array.from(MUSICS.keys());
    let musicString = MUSICS.get(musicNames[index]);
    //剔除空格
    musicString = musicString.replace(/ /g, '');
    //计算所有小节
    let strArrays = musicString.split('_');
    let prefix = strArrays[0];
    let theme = strArrays[1];
    var themeSections = theme.split('|');

    //根据起始节和结束节拼接要发送的音乐串
    let startSection = this.data.startSection;
    let endSection = this.data.endSection;
    //拼接前缀
    let oldbpm = prefix.split(',')[1];
    var final = prefix.replace(oldbpm, this.data.musicBpm[this.data.bpm])+'_';
    //拼接主题曲
    for(var i=startSection; i<=endSection; i+=1){
      final += themeSections[i]+'|';
    }
    final = final.substring(0, final.length - 1);

    //拼接伴奏
    for(var a=2; a<strArrays.length; a++){
      let accompany = strArrays[a];
      var accompanySections = accompany.split('|');
      final += '_';
      for (var i = startSection; i <= endSection; i += 1) {
        final += accompanySections[i] + '|';
      }
      final = final.substring(0, final.length - 1);
    }
    console.log('发送:', final);
    totalLen = final.length + 1;
    this.sendMessage(final+'#');
  },
  onCheckBoxChange(data){
    if(data.detail.value.length==0){
      console.log('关闭循环播放');
      this.data.loop = false;
    }else{
      console.log('打开循环播放');
      this.data.loop = true;
    }
  },
  closeBluetoothAdapter() {
    //关闭连接的设备
    this.closeBLEConnection();
    //关闭蓝牙
    wx.closeBluetoothAdapter()
    this._discoveryStarted = false;
    find_time = 0;
    this.setData({
      writeId: null,
      status: '蓝牙已关闭'
    });
  },
})
