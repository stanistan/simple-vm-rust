# -*- mode: ruby -*-
# vi: set ft=ruby :

# This is specifically here because I'm rumming MacOS
# and doing profiling and generating flamegraphs (and things)
# is very painful.
#
# So instead of messing around with docker (which I tried) and had
# its issues, I'm instead running all of that stuff in a VM,
# and this is the simplest way for me to set one up.
Vagrant.configure("2") do |config|
  config.vm.box = "hashicorp/precise64"
  config.vm.provision "shell", inline: <<-SHELL
    apt-get update
    apt-get install -y curl linux-tools make
    su vagrant -c 'curl https://sh.rustup.rs -sSf | sh -s -- -y'
    rustup default nightly
  SHELL
end
